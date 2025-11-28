use std::path::{Path, PathBuf};

use anyhow::Result;
use libcontainer::{oci_spec::runtime::{LinuxBuilder, LinuxIdMappingBuilder, LinuxNamespace, LinuxNamespaceBuilder, LinuxNamespaceType, Mount, Spec}};

pub struct SysUserParms {
    pub uid: u32,
    pub gid: u32,
}

pub fn get_rootless(sys_user: &SysUserParms) -> Result<Spec> {
    // Remove network and user namespace from the default spec
    let mut namespaces: Vec<LinuxNamespace> =
        libcontainer::oci_spec::runtime::get_default_namespaces()
            .into_iter()
            .filter(|ns| {
                ns.typ() != LinuxNamespaceType::Network && ns.typ() != LinuxNamespaceType::User
            })
            .collect();

    // Add user namespace
    namespaces.push(
        LinuxNamespaceBuilder::default()
            .typ(LinuxNamespaceType::User)
            .build()?,
    );

    let linux = LinuxBuilder::default()
        .namespaces(namespaces)
        .uid_mappings(vec![
            LinuxIdMappingBuilder::default()
                .host_id(sys_user.uid)
                .container_id(0_u32)
                .size(1_u32)
                .build()?,
        ])
        .gid_mappings(vec![
            LinuxIdMappingBuilder::default()
                .host_id(sys_user.gid)
                .container_id(0_u32)
                .size(1_u32)
                .build()?,
        ])
        .build()?;

    // Prepare the mounts

    let mut mounts: Vec<Mount> = libcontainer::oci_spec::runtime::get_default_mounts();
    for mount in &mut mounts {
        if mount.destination().eq(Path::new("/sys")) {
            mount
                .set_source(Some(PathBuf::from("/sys")))
                .set_typ(Some(String::from("none")))
                .set_options(Some(vec![
                    "rbind".to_string(),
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "ro".to_string(),
                ]));
        } else {
            let options: Vec<String> = mount
                .options()
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .filter(|&o| !o.starts_with("gid=") && !o.starts_with("uid="))
                .map(|o| o.to_string())
                .collect();
            mount.set_options(Some(options));
        }
    }

    let mut spec = get_default()?;
    spec.set_linux(Some(linux)).set_mounts(Some(mounts));
    Ok(spec)
}

pub fn get_default() -> Result<Spec> {
    Ok(Spec::default())
}
