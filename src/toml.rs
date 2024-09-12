use crate::context;

use std::{path, path::Path};

use anyhow::{bail, Context};
use cargo_util_schemas::core::GitReference;
use fs_err as fs;
use pathdiff::diff_paths;

pub enum Operation<'a> {
    Add {
        registry: &'a str,
        name: &'a str,
        mode: &'a context::Mode,
    },
    Remove {
        name: &'a str,
    },
}

pub fn patch_manifest(
    working_dir: &Path,
    manifest: &str,
    manifest_directory: &Path,
    op: Operation,
) -> anyhow::Result<String> {
    let mut manifest: toml_edit::DocumentMut = manifest
        .parse()
        .context("patch manifest contains invalid toml")?;

    let manifest_table = manifest.as_table_mut();

    match op {
        Operation::Add {
            registry,
            name,
            mode,
        } => add_patch_to_manifest(
            working_dir,
            manifest_table,
            manifest_directory,
            registry,
            name,
            mode,
        )?,
        Operation::Remove { name } => remove_patch_from_manifest(manifest_table, name)?,
    }

    Ok(manifest.to_string())
}

fn add_patch_to_manifest(
    working_dir: &Path,
    manifest_table: &mut toml_edit::Table,
    manifest_directory: &Path,
    registry: &str,
    name: &str,
    mode: &context::Mode,
) -> anyhow::Result<()> {
    let patch_table = create_subtable(manifest_table, "patch", true)?;
    let registry_table = create_subtable(patch_table, registry, false)?;

    toml_edit::Table::insert(
        registry_table,
        name,
        source(working_dir, manifest_directory, mode),
    );

    Ok(())
}

fn remove_patch_from_manifest(
    manifest_table: &mut toml_edit::Table,
    name: &str,
) -> anyhow::Result<()> {
    if let Some(patch_table) = manifest_table.get_mut("patch") {
        let mut to_remove = vec![];
        let patch_table = patch_table.as_table_mut().unwrap();
        for (register_name, register) in patch_table.iter_mut() {
            let register_table = register.as_table_mut().unwrap();
            register_table.remove(name);

            if register_table.is_empty() {
                to_remove.push(register_name.to_string());
            }
        }

        // TODO: somehow it removes the comment in the manifest file -> see test
        //       Removes a comment in the final toml file when using the tool as well
        //       Maybe it thinks the comment refers to the table.
        //       Reason: sees a comment before a table as a decor which belongs to it
        //       Solution: don't remove if there is any comment in front of the table?
        //       Solution2: toml_edit should only take direct attached comments as prefix?
        for register_name in to_remove {
            dbg!(patch_table.remove(register_name.as_str()));
        }
    }

    Ok(())
}

fn source(working_dir: &Path, manifest_directory: &Path, mode: &context::Mode) -> toml_edit::Item {
    let source = match mode {
        context::Mode::Path(relative_path) => {
            let attempt_to_canonicalize =
                |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

            let [manifest_directory, working_dir] =
                [manifest_directory, working_dir].map(attempt_to_canonicalize);

            let path = if manifest_directory != working_dir {
                diff_paths(
                    path::absolute(working_dir.join(relative_path)).unwrap(),
                    path::absolute(manifest_directory).unwrap(),
                )
                .expect("both paths are absolute")
            } else {
                relative_path.into()
            };

            format!("{{ path = \"{}\" }}", path.display())
        }
        context::Mode::Git { url, reference } => {
            let reference = match reference {
                GitReference::DefaultBranch => String::new(),
                GitReference::Tag(tag) => format!(", tag = \"{tag}\""),
                GitReference::Rev(rev) => format!(", rev = \"{rev}\""),
                GitReference::Branch(branch) => format!(", branch = \"{branch}\""),
            };

            format!("{{ git = \"{url}\"{reference} }}")
        }
    };

    let Ok(new_patch) = source.parse::<toml_edit::Item>() else {
        todo!("We haven't escaped anything, so we can't be sure this will parse")
    };

    new_patch
}

fn create_subtable<'a>(
    table: &'a mut toml_edit::Table,
    name: &str,
    dotted: bool,
) -> anyhow::Result<&'a mut toml_edit::Table> {
    let existing = &mut table[name];

    if existing.is_none() {
        // If the table does not exist, create it
        *existing = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // TODO: in the future we may be able to do cool things with miette
    let _span = existing.span();

    let Some(subtable) = existing.as_table_mut() else {
        bail!("{name} already exists but is not a table")
    };

    subtable.set_dotted(dotted);

    Ok(subtable)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_patch_manifest() {
        let manifest = r###"[package]
name = "package-name"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[patch.crates-io]
test = { git = "https://github.com/test/test.git" }

[patch.github]
test1 = { git = "https://github.com/test/test1.git" }
"###;

        let manifest_after_adding = patch_manifest(
            Path::new("/path/to/working/dir/"),
            &manifest,
            Path::new("/path/to/working/dir/"),
            Operation::Add {
                registry: "crates-io",
                name: "test2",
                mode: &context::Mode::Path("/path/to/local/crate/test2".into()),
            },
        )
        .unwrap();

        insta::assert_toml_snapshot!(manifest_after_adding, @r###"
        '''
        [package]
        name = "package-name"
        version = "0.1.0"
        edition = "2021"

        # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

        [patch.crates-io]
        test = { git = "https://github.com/test/test.git" }
        test2 = { path = "/path/to/local/crate/test2" }

        [patch.github]
        test1 = { git = "https://github.com/test/test1.git" }
        '''
        "###);

        let manifest_after_removing = patch_manifest(
            Path::new("/path/to/working/dir/"),
            &manifest,
            Path::new("/path/to/working/dir/"),
            Operation::Remove { name: "test" },
        )
        .unwrap();

        insta::assert_toml_snapshot!(manifest_after_removing, @r###"
        '''
        [package]
        name = "package-name"
        version = "0.1.0"
        edition = "2021"

        # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
        
        [patch.github]
        test1 = { git = "https://github.com/test/test1.git" }
        '''
        "###);
    }
}
