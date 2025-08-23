//! Unit tests for SMS module

#[cfg(test)]
pub mod sms_service_tests;
#[cfg(test)]
pub mod mock_sms_tests;
#[cfg(test)]
pub mod create_service_tests;
#[cfg(all(test, feature = "twilio-sms"))]
pub mod twilio_tests;