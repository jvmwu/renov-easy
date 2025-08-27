//! Distributed attack detection service
//!
//! This service analyzes authentication patterns to detect and prevent
//! distributed attacks such as credential stuffing and botnet attacks.

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use tracing::{warn, error};
use ipnetwork::{Ipv4Network, Ipv6Network};

use crate::domain::entities::audit::{AuditLog, AuditEventType};
use crate::errors::{DomainError, DomainResult};
use crate::repositories::AuditLogRepository;

/// Configuration for attack detection
#[derive(Debug, Clone)]
pub struct AttackDetectorConfig {
    /// Time window for analysis (minutes)
    pub analysis_window_minutes: i64,
    /// Threshold for unique IPs attacking same target
    pub ip_diversity_threshold: usize,
    /// Threshold for attacks from same subnet
    pub subnet_attack_threshold: usize,
    /// Threshold for rapid IP rotation (attacks per minute)
    pub ip_rotation_velocity: f64,
    /// Suspicious subnet mask (e.g., /24 for IPv4)
    pub ipv4_subnet_mask: u8,
    pub ipv6_subnet_mask: u8,
    /// Enable geographic anomaly detection
    pub enable_geo_detection: bool,
}

impl Default for AttackDetectorConfig {
    fn default() -> Self {
        Self {
            analysis_window_minutes: 10,
            ip_diversity_threshold: 5,     // 5+ different IPs attacking same target
            subnet_attack_threshold: 3,    // 3+ IPs from same subnet
            ip_rotation_velocity: 2.0,     // 2+ different IPs per minute
            ipv4_subnet_mask: 24,          // /24 subnet
            ipv6_subnet_mask: 48,          // /48 subnet
            enable_geo_detection: false,   // Disabled by default (requires GeoIP)
        }
    }
}

/// Attack detection result
#[derive(Debug, Clone)]
pub struct AttackDetectionResult {
    /// Whether a distributed attack is detected
    pub is_attack_detected: bool,
    /// Attack pattern type
    pub attack_pattern: Option<AttackPattern>,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
    /// Suspicious IPs identified
    pub suspicious_ips: Vec<String>,
    /// Targeted phone numbers
    pub targeted_phones: Vec<String>,
    /// Recommended action
    pub recommended_action: RecommendedAction,
    /// Detailed analysis
    pub analysis_details: String,
}

/// Types of attack patterns
#[derive(Debug, Clone, PartialEq)]
pub enum AttackPattern {
    /// Multiple IPs targeting same accounts
    CredentialStuffing,
    /// IPs from same subnet attacking
    SubnetAttack,
    /// Rapid rotation of IPs
    IpRotation,
    /// Geographic anomaly detected
    GeographicAnomaly,
    /// Mixed pattern attack
    MixedPattern,
}

/// Recommended actions for detected attacks
#[derive(Debug, Clone)]
pub enum RecommendedAction {
    /// No action needed
    None,
    /// Enable CAPTCHA
    EnableCaptcha,
    /// Block IP subnet
    BlockSubnet(String),
    /// Temporary system lockdown
    SystemLockdown,
    /// Alert administrators
    AlertAdmins,
}

/// Service for detecting distributed attacks
pub struct AttackDetector<A>
where
    A: AuditLogRepository,
{
    /// Audit log repository for querying events
    audit_repository: Arc<A>,
    /// Configuration
    config: AttackDetectorConfig,
}

impl<A> AttackDetector<A>
where
    A: AuditLogRepository,
{
    /// Create new attack detector
    pub fn new(audit_repository: Arc<A>, config: AttackDetectorConfig) -> Self {
        Self {
            audit_repository,
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(audit_repository: Arc<A>) -> Self {
        Self::new(audit_repository, AttackDetectorConfig::default())
    }

    /// Detect distributed attack patterns
    pub async fn detect_attack(&self) -> DomainResult<AttackDetectionResult> {
        let since = Utc::now() - Duration::minutes(self.config.analysis_window_minutes);

        // Query recent authentication events from audit log
        let events = self.get_recent_auth_events(since).await?;

        if events.is_empty() {
            return Ok(AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "No recent authentication events".to_string(),
            });
        }

        // Analyze patterns
        let credential_stuffing = self.detect_credential_stuffing(&events);
        let subnet_attack = self.detect_subnet_attack(&events);
        let ip_rotation = self.detect_ip_rotation(&events, since);

        // Combine results
        self.combine_detection_results(credential_stuffing, subnet_attack, ip_rotation)
    }

    /// Detect credential stuffing pattern (multiple IPs targeting same accounts)
    fn detect_credential_stuffing(&self, events: &[AuditLog]) -> AttackDetectionResult {
        let mut phone_to_ips: HashMap<String, HashSet<String>> = HashMap::new();

        for event in events {
            if let Some(phone) = &event.phone_masked {
                phone_to_ips
                    .entry(phone.clone())
                    .or_insert_with(HashSet::new)
                    .insert(event.ip_address.clone());
            }
        }

        // Find phones targeted by multiple IPs
        let suspicious_targets: Vec<(String, Vec<String>)> = phone_to_ips
            .into_iter()
            .filter(|(_, ips)| ips.len() >= self.config.ip_diversity_threshold)
            .map(|(phone, ips)| (phone, ips.into_iter().collect()))
            .collect();

        if !suspicious_targets.is_empty() {
            let all_ips: HashSet<String> = suspicious_targets
                .iter()
                .flat_map(|(_, ips)| ips.clone())
                .collect();

            let confidence = (suspicious_targets.len() as f64 / 10.0).min(0.9);

            warn!(
                pattern = "credential_stuffing",
                targets = suspicious_targets.len(),
                unique_ips = all_ips.len(),
                "Potential credential stuffing attack detected"
            );

            AttackDetectionResult {
                is_attack_detected: true,
                attack_pattern: Some(AttackPattern::CredentialStuffing),
                confidence_score: confidence,
                suspicious_ips: all_ips.into_iter().collect(),
                targeted_phones: suspicious_targets.iter().map(|(p, _)| p.clone()).collect(),
                recommended_action: RecommendedAction::EnableCaptcha,
                analysis_details: format!(
                    "Detected {} phone numbers targeted by multiple IPs (threshold: {})",
                    suspicious_targets.len(), self.config.ip_diversity_threshold
                ),
            }
        } else {
            AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "No credential stuffing pattern detected".to_string(),
            }
        }
    }

    /// Detect subnet attack pattern (multiple IPs from same subnet)
    fn detect_subnet_attack(&self, events: &[AuditLog]) -> AttackDetectionResult {
        let mut subnet_counts: HashMap<String, HashSet<String>> = HashMap::new();

        for event in events {
            if let Ok(ip) = event.ip_address.parse::<IpAddr>() {
                let subnet = self.get_subnet_for_ip(&ip);
                subnet_counts
                    .entry(subnet.clone())
                    .or_insert_with(HashSet::new)
                    .insert(event.ip_address.clone());
            }
        }

        // Find subnets with multiple attacking IPs
        let suspicious_subnets: Vec<(String, Vec<String>)> = subnet_counts
            .into_iter()
            .filter(|(_, ips)| ips.len() >= self.config.subnet_attack_threshold)
            .map(|(subnet, ips)| (subnet, ips.into_iter().collect()))
            .collect();

        if !suspicious_subnets.is_empty() {
            let primary_subnet = &suspicious_subnets[0];
            let confidence = (suspicious_subnets[0].1.len() as f64 / 10.0).min(0.95);

            warn!(
                pattern = "subnet_attack",
                subnet = primary_subnet.0,
                ip_count = primary_subnet.1.len(),
                "Potential subnet-based attack detected"
            );

            AttackDetectionResult {
                is_attack_detected: true,
                attack_pattern: Some(AttackPattern::SubnetAttack),
                confidence_score: confidence,
                suspicious_ips: primary_subnet.1.clone(),
                targeted_phones: vec![],
                recommended_action: RecommendedAction::BlockSubnet(primary_subnet.0.clone()),
                analysis_details: format!(
                    "Detected {} IPs from subnet {} (threshold: {})",
                    primary_subnet.1.len(), primary_subnet.0, self.config.subnet_attack_threshold
                ),
            }
        } else {
            AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "No subnet attack pattern detected".to_string(),
            }
        }
    }

    /// Detect rapid IP rotation pattern
    fn detect_ip_rotation(&self, events: &[AuditLog], since: DateTime<Utc>) -> AttackDetectionResult {
        let time_window = (Utc::now() - since).num_minutes() as f64;
        if time_window == 0.0 {
            return AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "Time window too small for rotation detection".to_string(),
            };
        }

        let unique_ips: HashSet<String> = events
            .iter()
            .map(|e| e.ip_address.clone())
            .collect();

        let rotation_velocity = unique_ips.len() as f64 / time_window;

        if rotation_velocity >= self.config.ip_rotation_velocity {
            let confidence = (rotation_velocity / 10.0).min(0.85);

            warn!(
                pattern = "ip_rotation",
                velocity = rotation_velocity,
                unique_ips = unique_ips.len(),
                "Rapid IP rotation detected"
            );

            AttackDetectionResult {
                is_attack_detected: true,
                attack_pattern: Some(AttackPattern::IpRotation),
                confidence_score: confidence,
                suspicious_ips: unique_ips.into_iter().collect(),
                targeted_phones: vec![],
                recommended_action: RecommendedAction::AlertAdmins,
                analysis_details: format!(
                    "Detected IP rotation velocity of {:.2} IPs/minute (threshold: {:.2})",
                    rotation_velocity, self.config.ip_rotation_velocity
                ),
            }
        } else {
            AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "No rapid IP rotation detected".to_string(),
            }
        }
    }

    /// Combine multiple detection results
    fn combine_detection_results(
        &self,
        credential_stuffing: AttackDetectionResult,
        subnet_attack: AttackDetectionResult,
        ip_rotation: AttackDetectionResult,
    ) -> DomainResult<AttackDetectionResult> {
        let mut detected_patterns = vec![];
        let mut all_suspicious_ips = HashSet::new();
        let mut all_targeted_phones = HashSet::new();
        let mut max_confidence: f64 = 0.0;
        let mut details = vec![];

        if credential_stuffing.is_attack_detected {
            detected_patterns.push(AttackPattern::CredentialStuffing);
            all_suspicious_ips.extend(credential_stuffing.suspicious_ips);
            all_targeted_phones.extend(credential_stuffing.targeted_phones);
            max_confidence = max_confidence.max(credential_stuffing.confidence_score);
            details.push(credential_stuffing.analysis_details);
        }

        if subnet_attack.is_attack_detected {
            detected_patterns.push(AttackPattern::SubnetAttack);
            all_suspicious_ips.extend(subnet_attack.suspicious_ips);
            max_confidence = max_confidence.max(subnet_attack.confidence_score);
            details.push(subnet_attack.analysis_details);
        }

        if ip_rotation.is_attack_detected {
            detected_patterns.push(AttackPattern::IpRotation);
            all_suspicious_ips.extend(ip_rotation.suspicious_ips);
            max_confidence = max_confidence.max(ip_rotation.confidence_score);
            details.push(ip_rotation.analysis_details);
        }

        if detected_patterns.is_empty() {
            return Ok(AttackDetectionResult {
                is_attack_detected: false,
                attack_pattern: None,
                confidence_score: 0.0,
                suspicious_ips: vec![],
                targeted_phones: vec![],
                recommended_action: RecommendedAction::None,
                analysis_details: "No attack patterns detected".to_string(),
            });
        }

        // Determine pattern and action
        let (pattern, action) = if detected_patterns.len() > 1 {
            // Multiple patterns indicate sophisticated attack
            max_confidence = (max_confidence * 1.2).min(0.99);
            (AttackPattern::MixedPattern, RecommendedAction::SystemLockdown)
        } else if subnet_attack.is_attack_detected {
            (AttackPattern::SubnetAttack, subnet_attack.recommended_action)
        } else if credential_stuffing.is_attack_detected {
            (AttackPattern::CredentialStuffing, RecommendedAction::EnableCaptcha)
        } else {
            (AttackPattern::IpRotation, RecommendedAction::AlertAdmins)
        };

        error!(
            pattern = ?pattern,
            confidence = max_confidence,
            suspicious_ips = all_suspicious_ips.len(),
            "Distributed attack detected!"
        );

        Ok(AttackDetectionResult {
            is_attack_detected: true,
            attack_pattern: Some(pattern),
            confidence_score: max_confidence,
            suspicious_ips: all_suspicious_ips.into_iter().collect(),
            targeted_phones: all_targeted_phones.into_iter().collect(),
            recommended_action: action,
            analysis_details: details.join("; "),
        })
    }

    /// Get subnet for an IP address
    fn get_subnet_for_ip(&self, ip: &IpAddr) -> String {
        match ip {
            IpAddr::V4(ipv4) => {
                let network = Ipv4Network::new(*ipv4, self.config.ipv4_subnet_mask)
                    .unwrap_or_else(|_| Ipv4Network::new(*ipv4, 32).unwrap());
                network.network().to_string()
            }
            IpAddr::V6(ipv6) => {
                let network = Ipv6Network::new(*ipv6, self.config.ipv6_subnet_mask)
                    .unwrap_or_else(|_| Ipv6Network::new(*ipv6, 128).unwrap());
                network.network().to_string()
            }
        }
    }

    /// Query recent authentication events from audit log
    async fn get_recent_auth_events(&self, since: DateTime<Utc>) -> DomainResult<Vec<AuditLog>> {
        // Query failed login attempts and rate limit violations
        let event_types = vec![
            AuditEventType::LoginFailure,
            AuditEventType::VerifyCodeFailure,
            AuditEventType::RateLimitExceeded,
            AuditEventType::AccountLocked,
        ];

        self.audit_repository
            .find_by_event_types(event_types, since, Utc::now(), Some(1000))
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to query audit logs: {}", e),
            })
    }

    /// Check if an IP is in a suspicious range
    pub fn is_suspicious_ip(&self, ip: &str) -> bool {
        // Check against known suspicious IP ranges
        // This is a simplified implementation
        if let Ok(addr) = ip.parse::<IpAddr>() {
            match addr {
                IpAddr::V4(ipv4) => {
                    // Check for common VPN/proxy ranges
                    let suspicious_ranges = [
                        "10.0.0.0/8",      // Private range (could be proxy)
                        "172.16.0.0/12",   // Private range
                        "192.168.0.0/16",  // Private range
                    ];

                    for range in &suspicious_ranges {
                        if let Ok(network) = range.parse::<Ipv4Network>() {
                            if network.contains(ipv4) {
                                return true;
                            }
                        }
                    }
                }
                IpAddr::V6(_) => {
                    // IPv6 suspicious range detection
                }
            }
        }
        false
    }

    /// Analyze attack patterns over time
    pub async fn analyze_attack_trends(&self, hours: i64) -> DomainResult<AttackTrendAnalysis> {
        let since = Utc::now() - Duration::hours(hours);
        let events = self.get_recent_auth_events(since).await?;

        // Group events by hour
        let mut hourly_counts: HashMap<String, usize> = HashMap::new();
        for event in &events {
            let hour_key = event.created_at.format("%Y-%m-%d %H:00").to_string();
            *hourly_counts.entry(hour_key).or_insert(0) += 1;
        }

        // Calculate trends
        let total_events = events.len();
        let unique_ips: HashSet<String> = events.iter().map(|e| e.ip_address.clone()).collect();
        let avg_events_per_hour = total_events as f64 / hours as f64;

        Ok(AttackTrendAnalysis {
            total_events,
            unique_ips: unique_ips.len(),
            average_events_per_hour: avg_events_per_hour,
            peak_hour: hourly_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(hour, _)| hour.clone()),
            hourly_distribution: hourly_counts,
        })
    }
}

/// Attack trend analysis results
#[derive(Debug, Clone)]
pub struct AttackTrendAnalysis {
    pub total_events: usize,
    pub unique_ips: usize,
    pub average_events_per_hour: f64,
    pub peak_hour: Option<String>,
    pub hourly_distribution: HashMap<String, usize>,
}
