use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

// FFI bindings for GNOME Online Accounts
// Note: This requires libgoa-1.0-dev to be installed
// For now, we'll provide a fallback implementation
#[cfg(feature = "goa")]
#[link(name = "goa-1.0")]
extern "C" {
    // GOA Client
    fn goa_client_new_sync(cancellable: *mut c_void, error: *mut *mut c_void) -> *mut c_void;
    fn goa_client_get_accounts(client: *mut c_void) -> *mut c_void;
    
    // GOA Object
    fn goa_object_get_account(object: *mut c_void) -> *mut c_void;
    
    // GOA Account
    fn goa_account_get_id(account: *mut c_void) -> *const c_char;
    fn goa_account_get_provider_name(account: *mut c_void) -> *const c_char;
    fn goa_account_get_identity(account: *mut c_void) -> *const c_char;
    fn goa_account_get_mail_disabled(account: *mut c_void) -> i32;
    
    // GOA Mail
    fn goa_object_get_mail(object: *mut c_void) -> *mut c_void;
    fn goa_mail_get_email_address(mail: *mut c_void) -> *const c_char;
    
    // GObject
    fn g_object_unref(object: *mut c_void);
    fn g_list_length(list: *mut c_void) -> u32;
    fn g_list_nth_data(list: *mut c_void, n: u32) -> *mut c_void;
    fn g_list_free(list: *mut c_void);
}

#[derive(Debug, Clone)]
pub struct EmailAccount {
    pub id: String,
    pub provider: String,
    pub email: String,
    pub identity: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_use_ssl: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_use_tls: bool,
}

#[cfg(feature = "goa")]
pub struct GoaClient {
    client: *mut c_void,
}

#[cfg(feature = "goa")]
impl GoaClient {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let client = goa_client_new_sync(ptr::null_mut(), ptr::null_mut());
            if client.is_null() {
                return Err("Failed to create GOA client".to_string());
            }
            Ok(GoaClient { client })
        }
    }
    
    pub fn get_email_accounts(&self) -> Result<Vec<EmailAccount>, String> {
        unsafe {
            let accounts_list = goa_client_get_accounts(self.client);
            if accounts_list.is_null() {
                return Ok(Vec::new());
            }
            
            let length = g_list_length(accounts_list);
            let mut email_accounts = Vec::new();
            
            for i in 0..length {
                let object_ptr = g_list_nth_data(accounts_list, i);
                if object_ptr.is_null() {
                    continue;
                }
                
                // Get the account from the object
                let account_ptr = goa_object_get_account(object_ptr);
                if account_ptr.is_null() {
                    continue;
                }
                
                // Check if this account has mail disabled
                let mail_disabled = goa_account_get_mail_disabled(account_ptr);
                if mail_disabled != 0 {
                    continue; // Skip if mail is disabled
                }
                
                // Get account details
                let id_ptr = goa_account_get_id(account_ptr);
                let provider_ptr = goa_account_get_provider_name(account_ptr);
                let identity_ptr = goa_account_get_identity(account_ptr);
                
                if id_ptr.is_null() || provider_ptr.is_null() || identity_ptr.is_null() {
                    continue;
                }
                
                let id = CStr::from_ptr(id_ptr).to_string_lossy().to_string();
                let provider = CStr::from_ptr(provider_ptr).to_string_lossy().to_string();
                let identity = CStr::from_ptr(identity_ptr).to_string_lossy().to_string();
                
                // Get mail settings
                let mail_ptr = goa_object_get_mail(object_ptr);
                if mail_ptr.is_null() {
                    continue;
                }
                
                let email_ptr = goa_mail_get_email_address(mail_ptr);
                if email_ptr.is_null() {
                    continue;
                }
                
                let email = CStr::from_ptr(email_ptr).to_string_lossy().to_string();
                
                // Use default settings based on provider
                let (imap_host, imap_port, imap_use_ssl, smtp_host, smtp_port, smtp_use_tls) = match provider.as_str() {
                    "google" => ("imap.gmail.com", 993, true, "smtp.gmail.com", 587, true),
                    "outlook" | "hotmail" => ("outlook.office365.com", 993, true, "smtp-mail.outlook.com", 587, true),
                    "yahoo" => ("imap.mail.yahoo.com", 993, true, "smtp.mail.yahoo.com", 587, true),
                    _ => ("imap.gmail.com", 993, true, "smtp.gmail.com", 587, true), // Default to Gmail settings
                };
                
                let account = EmailAccount {
                    id,
                    provider,
                    email,
                    identity,
                    imap_host: imap_host.to_string(),
                    imap_port,
                    imap_use_ssl,
                    smtp_host: smtp_host.to_string(),
                    smtp_port,
                    smtp_use_tls,
                };
                
                email_accounts.push(account);
            }
            
            g_list_free(accounts_list);
            Ok(email_accounts)
        }
    }
}

#[cfg(feature = "goa")]
impl Drop for GoaClient {
    fn drop(&mut self) {
        unsafe {
            g_object_unref(self.client);
        }
    }
}

// Safe wrapper function to get GNOME email accounts
#[cfg(feature = "goa")]
pub fn get_gnome_email_accounts() -> Result<Vec<EmailAccount>, String> {
    let client = GoaClient::new()?;
    client.get_email_accounts()
}

// Fallback implementation when GOA is not available
#[cfg(not(feature = "goa"))]
pub fn get_gnome_email_accounts() -> Result<Vec<EmailAccount>, String> {
    Err("GNOME Online Accounts library not available. Please install libgoa-1.0-dev and enable the 'goa' feature.".to_string())
}
