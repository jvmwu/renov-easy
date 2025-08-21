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

#### 1. User Authentication & Security
- **Passwordless Authentication**: SMS-based OTP verification with 5-minute expiry
- **Multi-Role System**: Customers, workers, admin with distinct permissions
- **Session Management**: JWT tokens (15min access, 30-day refresh)
- **Security Features**: Rate limiting (3 attempts/hour), audit logging, device tracking
- **Profile Verification**: Real-name authentication for workers, optional for customers

#### 2. Customer Features

**Order Management**
- **Smart Order Creation**: Multi-photo upload, voice-to-text description, urgency flags
- **Budget Ranges**: 4 tiers (1-5万, 5-10万, 10-20万, 20万+) with smart recommendations
- **Order Tracking**: Real-time status updates with milestone notifications
- **Quality Assurance**: Photo-based progress verification, completion checklists

**Discovery & Matching**
- **Location-Based Search**: 1-50km radius with heatmap visualization
- **Advanced Filtering**: Skills, ratings, price, availability, response time
- **Smart Recommendations**: AI-powered worker matching based on job requirements
- **Portfolio Browsing**: Before/after photos, certifications, customer reviews

**Communication & Payments**
- **Real-Time Chat**: Text, voice messages, photo sharing, location sharing
- **Video Consultation**: Schedule video calls for project discussion
- **Quote Management**: Receive and compare multiple quotes
- **Escrow Protection**: Secure payment holding (future phase)

#### 3. Worker Features

**Business Management**
- **Smart Job Discovery**: Map-based job visualization with distance calculation
- **Bid Management**: Competitive bidding with quote templates
- **Schedule Optimization**: Calendar integration, conflict detection
- **Team Management**: Add team members, assign tasks (future)

**Professional Profile**
- **Certification System**: Upload licenses, skill certificates with verification
- **Portfolio Builder**: Categorized project galleries with descriptions
- **Skill Tags**: 50+ predefined skills with proficiency levels
- **Service Areas**: Define multiple service zones with different rates

**Financial Tools**
- **Income Analytics**: Daily/weekly/monthly reports with charts
- **Expense Tracking**: Material costs, travel expenses, team wages
- **Tax Reports**: Automated income summaries for tax filing
- **Payment Methods**: Bank transfer, digital wallets (WeChat/Alipay)

#### 4. Service Categories (Detailed)
- **Kitchen Renovation**: Cabinet installation, countertops, appliances, plumbing
- **Bathroom Renovation**: Waterproofing, tiling, fixtures, ventilation
- **Living Room Renovation**: Flooring, painting, lighting, entertainment systems
- **Bedroom Renovation**: Wardrobes, flooring, insulation, air conditioning
- **Full House Renovation**: Complete redesign, structural changes, permits
- **Small Repairs**: Electrical fixes, plumbing leaks, door/window repairs, painting touch-ups

#### 5. Trust & Safety Systems
- **Rating System**: 4-dimension ratings (quality, timeliness, communication, value)
- **Dispute Resolution**: In-app mediation, evidence collection, arbitration
- **Insurance Integration**: Liability coverage verification (future)
- **Background Checks**: Criminal record verification for workers

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

## Product Roadmap & Milestones

### Phase 1: Foundation (Weeks 1-4) ✅ 
**Goal**: Establish robust backend infrastructure
- Core authentication system with SMS verification
- User management and role-based access control
- Database schema and migration framework
- RESTful API foundation with security middleware
- Audit logging and monitoring infrastructure

### Phase 2: Core Marketplace (Weeks 5-10) 🚧
**Goal**: Enable basic job posting and matching
- Order creation and management system
- Location-based search with Google Maps integration
- Worker profile and portfolio management
- Basic matching algorithm (distance + rating based)
- Order status workflow and notifications

### Phase 3: Communication & Trust (Weeks 11-16) 📝
**Goal**: Build user engagement and trust features
- Real-time chat system with WebSocket
- Bidirectional rating and review system
- Worker verification and certification management
- Push notification infrastructure
- Advanced search and filtering capabilities

### Phase 4: Mobile Applications (Weeks 17-24) 📝
**Goal**: Launch native mobile experiences
- FFI layer for Rust core integration
- iOS app with Swift/SwiftUI
- Android app with Kotlin/Jetpack Compose
- HarmonyOS app with ArkTS (optional)
- Cross-platform testing and optimization

### Phase 5: Growth & Monetization (Weeks 25-32) 🔒
**Goal**: Scale and generate revenue
- Payment gateway integration (Stripe/Alipay/WeChat Pay)
- Premium features for workers (boost visibility, priority matching)
- Advanced analytics and business intelligence
- AI-powered recommendations and pricing suggestions
- Multi-language expansion beyond Chinese/English