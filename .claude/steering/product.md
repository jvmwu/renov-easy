# Product Steering Document - RenovEasy

## Product Vision
RenovEasy (装修易) is a cross-platform mobile application that creates a seamless marketplace connecting homeowners with professional renovation workers for home maintenance and decoration services.

## Core Value Proposition
- **Primary Goal**: Facilitate convenient communication between customers and renovation workers
- **Key Focus**: Making small home repairs and decorations easy and accessible
- **Service Model**: On-demand, location-based matching for renovation services

## Target Market
- **Primary Market**: Australia
- **Language Support**: Bilingual (Chinese and English) with full internationalization
- **User Segments**:
  - Homeowners seeking renovation/repair services
  - Professional renovation workers and handymen

## Development Strategy
- **Phase 1 (Current)**: Rust-based backend development as shared foundation
- **Phase 2**: Native mobile app development for iOS, Android, and HarmonyOS
- **UI Reference**: HTML5 prototype in `/prototype/` serves as UI/UX blueprint
- **Development Model**: Solo developer, iterative approach

## Success Metrics
- Robust and scalable Rust backend architecture
- Smooth performance across all three platforms (iOS, Android, HarmonyOS)
- Beautiful and intuitive user interface (based on prototype)
- Excellent code performance and responsiveness
- User adoption and engagement rates
- Successful job completion rates

## Key Features

### Core Functionality
1. **User Authentication**
   - Phone number verification with SMS
   - User type selection (customer/worker)
   - Profile management

2. **Customer Features**
   - Post renovation requests with location
   - Browse nearby workers with ratings
   - View worker portfolios and certifications
   - Track job progress
   - Budget selection (1-5万, 5-10万, 10-20万, 20万+)
   - In-app chat with workers

3. **Worker Features**
   - View available jobs on map
   - Filter jobs by type and budget
   - Accept and manage jobs
   - Update job progress
   - Showcase portfolio and certifications
   - Income tracking
   - Client communication

4. **Service Categories**
   - Kitchen renovation
   - Bathroom renovation
   - Living room renovation
   - Bedroom renovation
   - Full house renovation
   - Small repairs and maintenance

### Essential Integrations
- **Google Maps**: Location services and mapping
- **SMS Service**: Phone verification
- **Push Notifications**: Real-time updates
- **Payment Processing**: Deferred to later phase

## Platform Strategy
- **Backend First**: Rust-based core business logic as foundation
- **Native Development**: Each platform uses native languages for optimal performance
- **Shared Business Logic**: Rust core accessed via FFI from all platforms
- **Consistent Experience**: Unified user experience across platforms while respecting platform conventions

## Regulatory Compliance
- Comply with Australian app store requirements
- Privacy and data protection standards
- No specific regulatory requirements currently identified

## Business Constraints
- Initial focus on core marketplace functionality
- Payment processing deferred to future phases
- No immediate expansion beyond home renovation services

## User Experience Principles
1. **Simplicity First**: Easy-to-use interface for both tech-savvy and non-technical users
2. **Trust Building**: Transparent worker profiles and verification
3. **Efficiency**: Quick job posting and acceptance process
4. **Communication**: Seamless in-app messaging between parties
5. **Localization**: Full support for Chinese and English languages

## Product Roadmap Priorities
1. **Phase 1 (Current)**: Rust backend with core business logic
2. **Phase 2**: Native app development with authentication and job posting
3. **Phase 3**: Chat functionality and enhanced profiles
4. **Phase 4**: Push notifications and SMS integration
5. **Phase 5**: Payment integration and advanced features