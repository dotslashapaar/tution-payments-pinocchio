

# Vire Protocol - Tuition Management System 

## Overview

Vire Protocol is a decentralized university tuition management system built on Solana using the Pinocchio framework. It facilitates university registration, subject creation, student enrollment, tuition payments, and degree verification through blockchain technology.

<img width="674" alt="415938590-d7b2a622-333c-4bcf-a779-3aab2183e1d3" src="https://github.com/user-attachments/assets/7baa3754-90fd-442c-9331-66b81d059220" />


## Features

- **Decentralized University Management**: Create and manage university entities on-chain
- **Subject Registration**: Universities can create courses with customizable tuition fees and semester structures
- **Student Enrollment**: Students can enroll in subjects and receive NFT certifications
- **Tuition Payment Processing**: Secure and transparent tuition payments via USDC
- **Degree Verification**: Time-locked credential system ensuring proper completion of educational requirements
- **NFT Certification**: Digital credentials represented as NFTs for both students and universities


## Architecture

### Core Accounts

1. **VireAccount**
    - Central administrator account for the entire system
    - Manages university count and transaction fee rates
    - Controls treasury account for protocol fees
2. **UniAccount**
    - Represents a registered university
    - Tracks subjects offered and enrolled students
    - Controlled by university administrators
3. **SubjectAccount**
    - Contains subject/course details
    - Defines tuition costs, semester requirements, and duration
4. **StudentAccount**
    - Tracks student enrollment and progress
    - Records enrollment time and completed semesters

### Key Instructions

1. **InitializeVire**
    - Creates the main administrative account
    - Sets up transaction fee rates for universities and students
2. **InitializeUni**
    - Registers a new university in the system
    - Creates university PDAs linked to the main Vire account
3. **AddSubjects**
    - Allows universities to add courses
    - Creates NFT collections for subject certification
    - Processes registration fees from universities to the protocol
4. **InitializeStudent**
    - Enrolls students in specific subjects
    - Mints NFT credentials that remain frozen until graduation
    - Creates student tracking accounts
5. **PayTutionFee**
    - Processes semester tuition payments
    - Distributes fees between university and protocol treasury
    - Updates student progress records
6. **UnStake**
    - Verifies degree completion requirements
    - Thaws student NFT credentials after successful verification
    - Transfers NFT ownership to student after graduation

## Technical Implementation

- Built on Solana blockchain using Pinocchio framework
- Uses PDAs (Program Derived Addresses) for secure account management
- Implements SPL token integration for USDC payments
- Utilizes NFTs for verifiable digital credentials
- Time-based verification using Solana's Clock sysvar


## Security Features

- Account validation through proper ownership checking
- Secure payment processing with frozen NFT credentials
- Time-locked degree verification system
- Authority checks for administrative actions


## Fees and Economics

- Universities pay protocol fees when registering new subjects
- Students pay tuition fees to universities plus a protocol fee
- Fees are customizable and stored in the main Vire Protocol Treasury


## Development

This project is implemented in Rust using the Pinocchio framework for Solana blockchain development. The codebase is organized into:

- State definitions (accounts)
- Instruction processing
- Context implementations for each instruction


---



