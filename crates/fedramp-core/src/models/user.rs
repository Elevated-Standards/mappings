// Modified: 2025-01-20

//! User models for FedRAMP compliance automation.
//!
//! This module defines data structures for user management,
//! authentication, and authorization.

use crate::types::{EntityId, Result, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    /// Unique user identifier
    pub id: EntityId,
    /// Username (unique)
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    /// Email address (unique)
    #[validate(email)]
    pub email: String,
    /// First name
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    /// Last name
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    /// User status
    pub status: UserStatus,
    /// User roles
    pub roles: Vec<UserRole>,
    /// User preferences
    pub preferences: UserPreferences,
    /// User profile
    pub profile: UserProfile,
    /// Password hash (not serialized)
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    /// Multi-factor authentication settings
    pub mfa: MfaSettings,
    /// Last login timestamp
    pub last_login: Option<Timestamp>,
    /// Failed login attempts
    pub failed_login_attempts: u32,
    /// Account locked until
    pub locked_until: Option<Timestamp>,
    /// Email verification status
    pub email_verified: bool,
    /// Email verification token
    #[serde(skip_serializing)]
    pub email_verification_token: Option<String>,
    /// Password reset token
    #[serde(skip_serializing)]
    pub password_reset_token: Option<String>,
    /// Password reset token expiry
    pub password_reset_expires: Option<Timestamp>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Created by user ID
    pub created_by: Option<EntityId>,
    /// Last updated by user ID
    pub updated_by: Option<EntityId>,
}

/// User account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserStatus {
    /// User account is active
    Active,
    /// User account is inactive
    Inactive,
    /// User account is suspended
    Suspended,
    /// User account is locked
    Locked,
    /// User account is pending activation
    PendingActivation,
    /// User account is archived
    Archived,
}

/// User roles in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    /// System administrator
    SystemAdmin,
    /// Security officer
    SecurityOfficer,
    /// Compliance manager
    ComplianceManager,
    /// Auditor
    Auditor,
    /// System owner
    SystemOwner,
    /// Authorizing official
    AuthorizingOfficial,
    /// Security control assessor
    SecurityControlAssessor,
    /// Information system security manager
    InformationSystemSecurityManager,
    /// Developer
    Developer,
    /// Analyst
    Analyst,
    /// Viewer (read-only)
    Viewer,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred language
    pub language: String,
    /// Preferred timezone
    pub timezone: String,
    /// Date format preference
    pub date_format: String,
    /// Time format preference
    pub time_format: String,
    /// Email notification settings
    pub email_notifications: EmailNotificationSettings,
    /// Dashboard preferences
    pub dashboard: DashboardPreferences,
    /// Theme preference
    pub theme: String,
}

/// Email notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotificationSettings {
    /// Enable email notifications
    pub enabled: bool,
    /// Notify on new findings
    pub new_findings: bool,
    /// Notify on finding updates
    pub finding_updates: bool,
    /// Notify on approaching deadlines
    pub approaching_deadlines: bool,
    /// Notify on overdue items
    pub overdue_items: bool,
    /// Notify on system changes
    pub system_changes: bool,
    /// Daily digest enabled
    pub daily_digest: bool,
    /// Weekly summary enabled
    pub weekly_summary: bool,
}

/// Dashboard preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPreferences {
    /// Default dashboard layout
    pub layout: String,
    /// Visible widgets
    pub widgets: Vec<String>,
    /// Widget positions
    pub widget_positions: HashMap<String, WidgetPosition>,
    /// Refresh interval in seconds
    pub refresh_interval: u32,
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Job title
    pub title: Option<String>,
    /// Department
    pub department: Option<String>,
    /// Organization
    pub organization: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Office location
    pub location: Option<String>,
    /// Manager user ID
    pub manager: Option<EntityId>,
    /// Direct reports
    pub direct_reports: Vec<EntityId>,
    /// Skills/certifications
    pub skills: Vec<String>,
    /// Bio/description
    pub bio: Option<String>,
    /// Profile picture URL
    pub avatar_url: Option<String>,
}

/// Multi-factor authentication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSettings {
    /// MFA enabled
    pub enabled: bool,
    /// MFA methods configured
    pub methods: Vec<MfaMethod>,
    /// Backup codes
    #[serde(skip_serializing)]
    pub backup_codes: Vec<String>,
    /// Recovery email
    pub recovery_email: Option<String>,
}

/// MFA method types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MfaMethod {
    /// Time-based one-time password (TOTP)
    Totp,
    /// SMS-based verification
    Sms,
    /// Email-based verification
    Email,
    /// Hardware security key
    SecurityKey,
    /// Backup codes
    BackupCodes,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: EntityId,
    /// Session token
    #[serde(skip_serializing)]
    pub token: String,
    /// Session creation time
    pub created_at: Timestamp,
    /// Session expiry time
    pub expires_at: Timestamp,
    /// Last activity time
    pub last_activity: Timestamp,
    /// IP address
    pub ip_address: String,
    /// User agent
    pub user_agent: String,
    /// Session active
    pub active: bool,
}

impl User {
    /// Create a new user
    pub fn new(
        username: String,
        email: String,
        first_name: String,
        last_name: String,
        created_by: Option<EntityId>,
    ) -> Self {
        let now = crate::utils::current_timestamp();
        let id = crate::utils::generate_uuid();

        Self {
            id,
            username,
            email,
            first_name,
            last_name,
            status: UserStatus::PendingActivation,
            roles: vec![UserRole::Viewer],
            preferences: UserPreferences::default(),
            profile: UserProfile::default(),
            password_hash: None,
            mfa: MfaSettings::default(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            email_verified: false,
            email_verification_token: None,
            password_reset_token: None,
            password_reset_expires: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Get user's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role(UserRole::SystemAdmin)
    }

    /// Check if user account is active
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active && !self.is_locked()
    }

    /// Check if user account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            crate::utils::current_timestamp() < locked_until
        } else {
            false
        }
    }

    /// Add a role to the user
    pub fn add_role(&mut self, role: UserRole) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
            self.updated_at = crate::utils::current_timestamp();
        }
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role: UserRole) {
        if let Some(pos) = self.roles.iter().position(|r| *r == role) {
            self.roles.remove(pos);
            self.updated_at = crate::utils::current_timestamp();
        }
    }

    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(crate::utils::current_timestamp());
        self.failed_login_attempts = 0;
        self.updated_at = crate::utils::current_timestamp();
    }

    /// Increment failed login attempts
    pub fn increment_failed_login(&mut self) {
        self.failed_login_attempts += 1;
        self.updated_at = crate::utils::current_timestamp();

        // Lock account after 5 failed attempts for 30 minutes
        if self.failed_login_attempts >= 5 {
            let lock_duration = chrono::Duration::minutes(30);
            self.locked_until = Some(crate::utils::current_timestamp() + lock_duration);
            self.status = UserStatus::Locked;
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "24h".to_string(),
            email_notifications: EmailNotificationSettings::default(),
            dashboard: DashboardPreferences::default(),
            theme: "light".to_string(),
        }
    }
}

impl Default for EmailNotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            new_findings: true,
            finding_updates: true,
            approaching_deadlines: true,
            overdue_items: true,
            system_changes: false,
            daily_digest: false,
            weekly_summary: true,
        }
    }
}

impl Default for DashboardPreferences {
    fn default() -> Self {
        Self {
            layout: "default".to_string(),
            widgets: vec![
                "compliance-overview".to_string(),
                "recent-findings".to_string(),
                "upcoming-deadlines".to_string(),
            ],
            widget_positions: HashMap::new(),
            refresh_interval: 300, // 5 minutes
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            title: None,
            department: None,
            organization: None,
            phone: None,
            location: None,
            manager: None,
            direct_reports: Vec::new(),
            skills: Vec::new(),
            bio: None,
            avatar_url: None,
        }
    }
}

impl Default for MfaSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            methods: Vec::new(),
            backup_codes: Vec::new(),
            recovery_email: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Test".to_string(),
            "User".to_string(),
            None,
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name(), "Test User");
        assert_eq!(user.status, UserStatus::PendingActivation);
        assert!(user.has_role(UserRole::Viewer));
        assert!(!user.is_admin());
        assert!(!user.is_active());
    }

    #[test]
    fn test_user_roles() {
        let mut user = User::new(
            "admin".to_string(),
            "admin@example.com".to_string(),
            "Admin".to_string(),
            "User".to_string(),
            None,
        );

        user.add_role(UserRole::SystemAdmin);
        assert!(user.has_role(UserRole::SystemAdmin));
        assert!(user.is_admin());

        user.remove_role(UserRole::Viewer);
        assert!(!user.has_role(UserRole::Viewer));
    }

    #[test]
    fn test_failed_login_attempts() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Test".to_string(),
            "User".to_string(),
            None,
        );

        // Simulate failed login attempts
        for _ in 0..5 {
            user.increment_failed_login();
        }

        assert_eq!(user.failed_login_attempts, 5);
        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.is_locked());
    }
}
