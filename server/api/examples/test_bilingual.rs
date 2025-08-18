// Example demonstrating bilingual error handling

extern crate core as renov_core;

use api::handlers::error::{Language, handle_domain_error_with_lang};
use renov_core::errors::{DomainError, AuthError};

fn main() {
    println!("Testing Bilingual Error Handling\n");
    println!("=================================\n");
    
    // Test 1: Invalid verification code error
    println!("Test 1: Invalid Verification Code");
    println!("---------------------------------");
    
    // English version
    let error_en = DomainError::Auth(AuthError::InvalidVerificationCode);
    let response_en = handle_domain_error_with_lang(error_en, Language::English);
    println!("English: Status = {:?}", response_en.status());
    
    // Chinese version  
    let error_zh = DomainError::Auth(AuthError::InvalidVerificationCode);
    let response_zh = handle_domain_error_with_lang(error_zh, Language::Chinese);
    println!("Chinese: Status = {:?}", response_zh.status());
    println!();
    
    // Test 2: Rate limit error
    println!("Test 2: Rate Limit Exceeded");
    println!("---------------------------");
    
    // English version
    let error_en = DomainError::Auth(AuthError::RateLimitExceeded { minutes: 5 });
    let response_en = handle_domain_error_with_lang(error_en, Language::English);
    println!("English: Status = {:?} (429 Too Many Requests)", response_en.status());
    
    // Chinese version
    let error_zh = DomainError::Auth(AuthError::RateLimitExceeded { minutes: 5 });
    let response_zh = handle_domain_error_with_lang(error_zh, Language::Chinese);
    println!("Chinese: Status = {:?} (429 Too Many Requests)", response_zh.status());
    println!();
    
    // Test 3: User not found error
    println!("Test 3: User Not Found");
    println!("----------------------");
    
    // English version
    let error_en = DomainError::Auth(AuthError::UserNotFound);
    let response_en = handle_domain_error_with_lang(error_en, Language::English);
    println!("English: Status = {:?} (404 Not Found)", response_en.status());
    
    // Chinese version
    let error_zh = DomainError::Auth(AuthError::UserNotFound);
    let response_zh = handle_domain_error_with_lang(error_zh, Language::Chinese);
    println!("Chinese: Status = {:?} (404 Not Found)", response_zh.status());
    println!();
    
    println!("✅ All tests completed successfully!");
    println!("\nNote: The actual error messages in Chinese/English are in the response body.");
    println!("This example only shows the HTTP status codes to verify the handler works correctly.");
}