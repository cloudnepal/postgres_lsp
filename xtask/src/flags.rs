#![allow(unreachable_pub)]

use crate::install::ClientOpt;

xflags::xflags! {
    src "./src/flags.rs"

    /// Run custom build command.
    cmd xtask {

        /// Install postgres_lsp server or editor plugin.
        cmd install {
            /// Install only VS Code plugin.
            optional --client
            /// One of 'code', 'code-exploration', 'code-insiders', 'codium', or 'code-oss'.
            optional --code-bin name: String

            /// Install only the language server.
            optional --server
        }
    }
}

// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct Xtask {
    pub subcommand: XtaskCmd,
}

#[derive(Debug)]
pub enum XtaskCmd {
    Install(Install),
}

#[derive(Debug)]
pub struct Install {
    pub client: bool,
    pub code_bin: Option<String>,
    pub server: bool,
}

impl Xtask {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end

impl Install {
    pub(crate) fn server(&self) -> Option<()> {
        if self.client && !self.server {
            return None;
        }
        Some(())
    }
    pub(crate) fn client(&self) -> Option<ClientOpt> {
        if !self.client && self.server {
            return None;
        }
        Some(ClientOpt { code_bin: self.code_bin.clone() })
    }
}