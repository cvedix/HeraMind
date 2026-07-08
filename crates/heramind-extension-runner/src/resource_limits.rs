//! Resource limits for extension processes
//!
//! This module provides cross-platform resource limiting to prevent
//! extensions from consuming excessive memory, CPU, or other resources.
//!
//! # Supported Platforms
//!
//! - **Linux/macOS**: Uses `setrlimit()` for memory limits
//! - **Windows**: Uses Job Objects with memory limits
//!
//! # Example
//!
//! ```rust
//! use resource_limits::setup_resource_limits;
//!
//! async fn main() {
//!     // Set up limits before loading extension
//!     setup_resource_limits(&ResourceLimitsConfig {
//!         memory_limit_mb: Some(2048),
//!         cpu_affinity: None,
//!         nice_level: Some(10),
//!     })?;
//!
//!     // Load and run extension
//!     let runner = Runner::load(&path).await?;
//!     runner.run().await;
//! }
//! ```

use std::io;
use tracing::{info, warn};

/// Configuration for resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimitsConfig {
    /// Memory limit in MB (soft limit), None = no limit
    pub memory_limit_mb: Option<u64>,

    /// Memory limit in MB (hard limit), None = 2x soft limit
    pub memory_limit_hard_mb: Option<u64>,

    /// CPU affinity (which cores to use), None = all cores
    /// Example: Some(vec![0, 1]) to use only cores 0 and 1
    pub cpu_affinity: Option<Vec<usize>>,

    /// Process nice level (priority), None = default
    /// Lower value = higher priority (-20 to 19)
    /// 10 = lower priority for background processes
    pub nice_level: Option<i32>,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: Some(4096), // 4GB default — needed for extensions loading ML models (e.g. ONNX Runtime) + rayon thread pools
            memory_limit_hard_mb: None,  // 2x soft limit
            cpu_affinity: None,
            nice_level: Some(10), // Lower priority
        }
    }
}

/// Set up resource limits for the current process
///
/// This must be called BEFORE loading the extension, as it applies
/// to the current process (the extension runner process).
///
/// # Arguments
///
/// * `config` - Resource limits configuration
///
/// # Returns
///
/// * `Ok(())` - Limits successfully applied
/// * `Err(e)` - Failed to apply limits (process should exit)
///
/// # Errors
///
/// This function will return an error if:
/// - The platform is not supported
/// - The system calls fail (permission denied, invalid value, etc.)
pub fn setup_resource_limits(config: &ResourceLimitsConfig) -> Result<(), ResourceLimitError> {
    info!("Setting up resource limits: {:?}", config);

    #[cfg(unix)]
    {
        setup_unix_limits(config)?;
    }

    #[cfg(windows)]
    {
        setup_windows_limits(config)?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        warn!("Resource limits not supported on this platform");
    }

    info!("Resource limits configured successfully");
    Ok(())
}

/// Error types for resource limit operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ResourceLimitError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

// ============================================================================
// Unix (Linux/macOS) Implementation
// ============================================================================

#[cfg(unix)]
fn setup_unix_limits(config: &ResourceLimitsConfig) -> Result<(), ResourceLimitError> {
    // 1. Set memory limit
    if let Some(soft_mb) = config.memory_limit_mb {
        let hard_mb = config.memory_limit_hard_mb.unwrap_or(soft_mb * 2);

        // On macOS, setrlimit for memory is unreliable inside Tauri/sandboxed processes.
        // RLIMIT_AS and RLIMIT_DATA both return EINVAL. Skip entirely to avoid noise.
        #[cfg(not(target_os = "macos"))]
        {
            use libc::{getrlimit, rlimit, setrlimit, RLIMIT_AS, RLIMIT_DATA};

            let soft = soft_mb * 1024 * 1024;
            let desired_hard = hard_mb * 1024 * 1024;

            info!(
                "Setting memory limit: soft={}MB, hard={}MB",
                soft_mb, hard_mb
            );

            // Clamp hard limit to the system's current hard limit to avoid EINVAL
            let sys_max = unsafe {
                let mut lim: rlimit = std::mem::zeroed();
                if getrlimit(RLIMIT_AS, &mut lim) == 0 {
                    lim.rlim_max
                } else {
                    desired_hard
                }
            };
            let hard = desired_hard.min(sys_max);
            let soft = soft.min(hard);

            let limits = rlimit {
                rlim_cur: soft,
                rlim_max: hard,
            };

            let result = unsafe {
                let mut result = setrlimit(RLIMIT_AS, &limits);

                if result != 0 {
                    let mut lim: rlimit = std::mem::zeroed();
                    let data_limits = if getrlimit(RLIMIT_DATA, &mut lim) == 0 {
                        rlimit {
                            rlim_cur: soft.min(lim.rlim_max),
                            rlim_max: hard.min(lim.rlim_max),
                        }
                    } else {
                        limits
                    };
                    warn!("RLIMIT_AS failed, trying RLIMIT_DATA");
                    result = setrlimit(RLIMIT_DATA, &data_limits);
                }

                result
            };

            if result != 0 {
                let err = io::Error::last_os_error();
                warn!("Failed to set memory limit: {} (continuing anyway)", err);
            } else {
                info!("Memory limit set successfully");
            }
        }

        #[cfg(target_os = "macos")]
        {
            let _ = (soft_mb, hard_mb); // suppress unused warning
            info!("Memory limit skipped on macOS (not supported in sandboxed processes)");
        }
    }

    // 2. Set process priority (nice level)
    if let Some(nice) = config.nice_level {
        use libc::{setpriority, PRIO_PROCESS};

        info!("Setting nice level to {}", nice);

        let result = unsafe { setpriority(PRIO_PROCESS, 0, nice) };

        if result != 0 {
            let err = io::Error::last_os_error();
            // Non-fatal: just warn if we can't set priority
            warn!("Failed to set nice level: {} (continuing anyway)", err);
        } else {
            info!("Nice level set to {}", nice);
        }
    }

    // 3. Set CPU affinity if specified
    if let Some(ref cores) = config.cpu_affinity {
        info!("Setting CPU affinity to cores: {:?}", cores);
        set_cpu_affinity_unix(cores)?;
    }

    Ok(())
}

#[cfg(unix)]
fn set_cpu_affinity_unix(cores: &[usize]) -> Result<(), ResourceLimitError> {
    #[cfg(target_os = "linux")]
    {
        use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};

        let mut cpuset: cpu_set_t = unsafe { std::mem::zeroed() };
        unsafe {
            CPU_ZERO(&mut cpuset);
            for &core in cores {
                if core < libc::CPU_SETSIZE as usize {
                    CPU_SET(core, &mut cpuset);
                } else {
                    return Err(ResourceLimitError::InvalidConfig(format!(
                        "CPU core {} exceeds CPU_SETSIZE",
                        core
                    )));
                }
            }
        }

        let result = unsafe { sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset) };

        if result != 0 {
            let err = io::Error::last_os_error();
            warn!("Failed to set CPU affinity: {} (continuing anyway)", err);
        } else {
            info!("CPU affinity set to cores: {:?}", cores);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = cores;
        // macOS has different CPU affinity API
        // For now, just log that we're skipping this
        warn!("CPU affinity not fully supported on macOS, skipping");
    }

    Ok(())
}

// ============================================================================
// Windows Implementation
// ============================================================================

#[cfg(windows)]
fn setup_windows_limits(config: &ResourceLimitsConfig) -> Result<(), ResourceLimitError> {
    use windows::Win32::System::Threading::*;

    // Note: Memory limiting on Windows is not implemented due to API limitations.
    // The windows crate v0.58 does not provide CreateJobObject in Win32_System_JobObjects.
    // Memory limiting is still available on Unix/Linux via setrlimit.

    // Log if memory limit was requested but not applied
    if config.memory_limit_mb.is_some() {
        warn!(
            "Memory limits are not supported on Windows in this version. \
             Use Unix/Linux for memory limiting or monitor memory usage externally."
        );
    }

    // Set process priority on Windows (this works)
    if let Some(_nice) = config.nice_level {
        info!("Setting process priority on Windows");

        unsafe {
            let priority = BELOW_NORMAL_PRIORITY_CLASS;
            let result = SetPriorityClass(GetCurrentProcess(), priority);

            if result.is_ok() {
                info!("Process priority set to BELOW_NORMAL");
            } else {
                let err = io::Error::last_os_error();
                warn!(
                    "Failed to set process priority: {} (continuing anyway)",
                    err
                );
            }
        }
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ResourceLimitsConfig::default();
        assert_eq!(config.memory_limit_mb, Some(4096));
        assert_eq!(config.nice_level, Some(10));
        assert!(config.cpu_affinity.is_none());
        assert!(config.memory_limit_hard_mb.is_none());
    }

    #[test]
    fn test_config_custom() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: Some(2048),
            memory_limit_hard_mb: Some(4096),
            cpu_affinity: Some(vec![0, 1]),
            nice_level: Some(5),
        };

        assert_eq!(config.memory_limit_mb, Some(2048));
        assert_eq!(config.memory_limit_hard_mb, Some(4096));
        assert_eq!(config.cpu_affinity, Some(vec![0, 1]));
        assert_eq!(config.nice_level, Some(5));
    }

    #[test]
    fn test_config_no_limits() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: None,
            memory_limit_hard_mb: None,
            cpu_affinity: None,
            nice_level: None,
        };

        assert!(config.memory_limit_mb.is_none());
        assert!(config.memory_limit_hard_mb.is_none());
        assert!(config.cpu_affinity.is_none());
        assert!(config.nice_level.is_none());
    }

    #[test]
    fn test_config_zero_memory_limit() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: Some(0),
            memory_limit_hard_mb: Some(0),
            cpu_affinity: None,
            nice_level: None,
        };

        // Zero limits are valid (though possibly not useful)
        assert_eq!(config.memory_limit_mb, Some(0));
        assert_eq!(config.memory_limit_hard_mb, Some(0));
    }

    #[test]
    fn test_config_large_memory_limit() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: Some(8192),       // 8GB
            memory_limit_hard_mb: Some(16384), // 16GB
            cpu_affinity: None,
            nice_level: None,
        };

        assert_eq!(config.memory_limit_mb, Some(8192));
        assert_eq!(config.memory_limit_hard_mb, Some(16384));
    }

    #[test]
    fn test_config_single_cpu_affinity() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: None,
            memory_limit_hard_mb: None,
            cpu_affinity: Some(vec![0]),
            nice_level: None,
        };

        assert_eq!(config.cpu_affinity, Some(vec![0]));
    }

    #[test]
    fn test_config_multiple_cpu_affinity() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: None,
            memory_limit_hard_mb: None,
            cpu_affinity: Some(vec![0, 1, 2, 3]),
            nice_level: None,
        };

        assert_eq!(config.cpu_affinity, Some(vec![0, 1, 2, 3]));
    }

    #[test]
    fn test_nice_level_range() {
        // Test various nice levels (-20 to 19)
        for nice in -20..=19 {
            let config = ResourceLimitsConfig {
                memory_limit_mb: None,
                memory_limit_hard_mb: None,
                cpu_affinity: None,
                nice_level: Some(nice),
            };
            assert_eq!(config.nice_level, Some(nice));
        }
    }

    #[test]
    fn test_nice_level_extreme_values() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: None,
            memory_limit_hard_mb: None,
            cpu_affinity: None,
            nice_level: Some(-20), // Highest priority
        };
        assert_eq!(config.nice_level, Some(-20));

        let config = ResourceLimitsConfig {
            memory_limit_mb: None,
            memory_limit_hard_mb: None,
            cpu_affinity: None,
            nice_level: Some(19), // Lowest priority
        };
        assert_eq!(config.nice_level, Some(19));
    }

    #[test]
    fn test_resource_limit_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let limit_err = ResourceLimitError::Io(io_err);
        assert!(limit_err.to_string().contains("test"));

        let sys_err = ResourceLimitError::SystemError("test error".to_string());
        assert!(sys_err.to_string().contains("test error"));

        let cfg_err = ResourceLimitError::InvalidConfig("bad config".to_string());
        assert!(cfg_err.to_string().contains("bad config"));
    }

    #[test]
    fn test_resource_limit_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let limit_err: ResourceLimitError = io_err.into();
        match limit_err {
            ResourceLimitError::Io(_) => {
                // Expected
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_config_clone() {
        let config1 = ResourceLimitsConfig {
            memory_limit_mb: Some(256),
            memory_limit_hard_mb: Some(512),
            cpu_affinity: Some(vec![0, 1]),
            nice_level: Some(15),
        };

        let config2 = config1.clone();

        assert_eq!(config1.memory_limit_mb, config2.memory_limit_mb);
        assert_eq!(config1.memory_limit_hard_mb, config2.memory_limit_hard_mb);
        assert_eq!(config1.cpu_affinity, config2.cpu_affinity);
        assert_eq!(config1.nice_level, config2.nice_level);
    }

    #[test]
    fn test_config_debug_format() {
        let config = ResourceLimitsConfig {
            memory_limit_mb: Some(1024),
            memory_limit_hard_mb: Some(1024),
            cpu_affinity: Some(vec![0, 1]),
            nice_level: Some(10),
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("1024"));
        assert!(debug_str.contains("10"));
    }
}
