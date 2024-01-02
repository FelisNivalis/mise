use std::fs::{create_dir_all, File};
use std::io::Write;

use eyre::Result;

use crate::config::Config;
use crate::hash::hash_to_str;

use crate::env;
use crate::toolset::ToolsetBuilder;

/// [internal] This is an internal command that writes an envrc file
/// for direnv to consume.
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, hide = true)]
pub struct Envrc {}

impl Envrc {
    pub fn run(self, config: &Config) -> Result<()> {
        let ts = ToolsetBuilder::new().build(config)?;

        let envrc_path = env::MISE_TMP_DIR
            .join("direnv")
            .join(hash_to_str(&env::current_dir()?) + ".envrc");

        // TODO: exit early if envrc_path exists and is up to date
        create_dir_all(envrc_path.parent().unwrap())?;
        let mut file = File::create(&envrc_path)?;

        writeln!(
            file,
            "### Do not edit. This was autogenerated by 'asdf direnv envrc' ###"
        )?;
        for cf in config.config_files.keys() {
            writeln!(file, "watch_file {}", cf.to_string_lossy())?;
        }
        for (k, v) in ts.env(config) {
            if k == "PATH" {
                writeln!(file, "PATH_add {}", v)?;
            } else {
                writeln!(
                    file,
                    "export {}={}",
                    shell_escape::unix::escape(k.into()),
                    shell_escape::unix::escape(v.into()),
                )?;
            }
        }
        for path in ts.list_paths().into_iter().rev() {
            writeln!(file, "PATH_add {}", path.to_string_lossy())?;
        }

        miseprintln!("{}", envrc_path.to_string_lossy());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{dirs, file};

    #[test]
    fn test_direnv_envrc() {
        assert_cli!("install");
        let stdout = assert_cli!("direnv", "envrc");
        let envrc = file::read_to_string(stdout.trim()).unwrap();
        let envrc = envrc.replace(dirs::HOME.to_string_lossy().as_ref(), "~");
        let envrc = envrc
            .lines()
            .filter(|l| !l.starts_with("export REMOTE_"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_display_snapshot!(envrc);
    }
}
