use anyhow::{Context, Result};
use handlebars::Handlebars;

use std::path::Path;
use std::process::Command;

use crate::filesystem::{Filesystem, RealFilesystem};

pub(crate) fn run_hook(
    location: &Path,
    cache_dir: &Path,
    handlebars: &Handlebars,
    variables: &crate::config::Variables,
) -> Result<()> {
    if !location.exists() {
        debug!("Hook file at {:?} missing", location);
        return Ok(());
    }

    let fs = &mut RealFilesystem::new(false);

    // Default to current location
    let mut script_file = location.into();

    // If it is templated, then render it into cache and run from there
    if fs
        .is_template(location)
        .context(format!("check whether {:?} is a template", location))?
    {
        script_file = cache_dir.join(location);
        if cfg!(windows) {
            script_file.set_extension("bat");
        }

        debug!("Rendering script {:?} -> {:?}", location, script_file);

        crate::actions::perform_template_deploy(
            location,
            &script_file,
            &std::env::temp_dir().join("dotter_temp").into(),
            fs,
            handlebars,
            variables,
        )
        .context("deploy script")?;
    }

    debug!("Running script file");
    let mut child = if cfg!(windows) {
        Command::new(script_file)
            .spawn()
            .context("spawn batch file")?
    } else {
        Command::new("sh")
            .arg(script_file)
            .spawn()
            .context("spawn shell")?
    };

    anyhow::ensure!(
        child.wait().context("wait for child shell")?.success(),
        "subshell returned error"
    );

    Ok(())
}
