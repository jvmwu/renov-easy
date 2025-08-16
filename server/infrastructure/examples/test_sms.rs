//! Example demonstrating the SMS service functionality

use infrastructure::sms::{SmsService, MockSmsService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a mock SMS service
    let sms_service = MockSmsService::new();
    
    println!("Testing SMS Service Implementation\n");
    
    // Test 1: Send a regular SMS
    println!("Test 1: Sending regular SMS");
    let result = sms_service.send_sms("+1234567890", "Hello from RenovEasy!").await;
    match result {
        Ok(message_id) => println!("✓ SMS sent successfully. Message ID: {}\n", message_id),
        Err(e) => println!("✗ Failed to send SMS: {}\n", e),
    }
    
    // Test 2: Send a verification code
    println!("Test 2: Sending verification code");
    let result = sms_service.send_verification_code("+9876543210", "123456").await;
    match result {
        Ok(message_id) => println!("✓ Verification code sent. Message ID: {}\n", message_id),
        Err(e) => println!("✗ Failed to send verification code: {}\n", e),
    }
    
    // Test 3: Invalid phone number
    println!("Test 3: Testing invalid phone number");
    let result = sms_service.send_sms("1234567890", "This should fail").await;
    match result {
        Ok(_) => println!("✗ Should have failed for invalid phone number\n"),
        Err(e) => println!("✓ Correctly rejected invalid number: {}\n", e),
    }
    
    // Test 4: Check service availability
    println!("Test 4: Checking service availability");
    let available = sms_service.is_available().await;
    println!("Service available: {}\n", available);
    
    // Test 5: Provider name
    println!("Test 5: Getting provider name");
    println!("Provider: {}\n", sms_service.provider_name());
    
    // Test 6: Message count
    println!("Test 6: Message counter");
    println!("Total messages sent: {}", sms_service.get_message_count());
    
    Ok(())
}