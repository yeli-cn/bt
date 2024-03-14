use derive_builder::Builder;
use log::{debug, info};
use std::path::{Path, PathBuf};

#[derive(Default, Builder, Debug, PartialEq, Eq)]
#[builder(setter(into))]
pub struct BangumiInfo {
    pub show_name: String,
    pub episode_name: Option<String>,
    pub display_name: Option<String>,
    pub season: u64,
    pub episode: u64,
    pub category: Option<String>,
}

impl BangumiInfo {
    pub fn folder_name(&self) -> String {
        String::from(format!("{}", self.show_name))
    }

    pub fn sub_folder_name(&self) -> String {
        String::from(format!("Season {}", self.season))
    }

    pub fn file_name(&self, extension: &str) -> String {
        assert!(extension.starts_with('.'), "extension must start with a dot");

        let mut file_name = String::new();

        file_name.push_str(&self.show_name.clone());
        file_name.push_str(&format!(" S{:02}E{:02}", self.season, self.episode));

        if let Some(ref episode_name) = self.episode_name {
            file_name.push_str(&format!(" {}", episode_name));
        }

        if let Some(ref display_name) = self.display_name {
            file_name.push_str(&format!(" {}", display_name));
        }

        file_name.push_str(extension);

        file_name
    }

    pub fn gen_path(&self, extension: &str) -> PathBuf {
        PathBuf::new()
            .join(self.folder_name())
            .join(self.sub_folder_name())
            .join(self.file_name(&format!(".{}", extension)))
    }
}

/// Link the file to the correct location.
/// e.g., if the `src_path` is `/download/Sousou no Frieren S01E01.mkv`,
/// and the `dst_folder` is `/media/TV`,
/// it should be linked to `/media/TV/Sousou no Frieren/Season 1/Sousou no Frieren S01E01.mkv`.
pub fn rename(info: &BangumiInfo, src_path: &Path, dst_folder: &Path) -> anyhow::Result<()> {
    if !src_path.exists() {
        return Err(anyhow::Error::msg("File does not exist"));
    }

    // TODO: support folder
    if !src_path.is_file() {
        return Err(anyhow::Error::msg("Unsupported file type"));
    }

    let extension = src_path
        .extension()
        .ok_or(anyhow::Error::msg(format!("File {} has no extension", &src_path.display())))?;

    let dst_path = dst_folder.join(info.gen_path(extension.to_str().unwrap()));

    link(src_path, &dst_path)?;

    Ok(())
}

pub fn link(src_path: &Path, dst_path: &Path) -> anyhow::Result<()> {
    info!("[renamer] Linking {} to {}", src_path.display(), dst_path.display());
    if !src_path.is_file() {
        return Err(anyhow::Error::msg("Only file type can be linked"));
    }

    let dst_parent = dst_path.parent().unwrap();
    if !dst_parent.exists() {
        std::fs::create_dir_all(dst_parent)?;
    }

    if dst_path.exists() {
        debug!("[renamer] File {} already linked", dst_path.display());
        return Ok(());
    }

    std::fs::hard_link(src_path, dst_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_path() {
        let bangumi_infos = vec![
            BangumiInfo {
                show_name: String::from("Sousou no Frieren"),
                season: 1,
                episode: 12,
                ..Default::default()
            },
            BangumiInfo {
                show_name: String::from("Sousou no Frieren"),
                episode_name: Some(String::from("冒险的终点")),
                display_name: None,
                season: 1,
                episode: 1,
                category: None,
            },
        ];

        let res_paths = vec![
            PathBuf::from("Sousou no Frieren/Season 1/Sousou no Frieren S01E12.mkv"),
            PathBuf::from("Sousou no Frieren/Season 1/Sousou no Frieren S01E01 冒险的终点.mkv"),
        ];

        for (info, res_path) in bangumi_infos.iter().zip(res_paths.iter()) {
            assert_eq!(info.gen_path("mkv"), *res_path);
        }
    }

    #[test]
    fn test_rename() {
        let src_path = Path::new("/tmp/Sousou no Frieren S01E00.mkv");

        std::fs::write(src_path, "test").unwrap();

        let dst_folder = Path::new("/tmp/TV");
        let bangumi_info = BangumiInfo {
            show_name: String::from("Sousou no Frieren"),
            season: 1,
            episode: 12,
            ..Default::default()
        };

        let dst_path = dst_folder.join("Sousou no Frieren/Season 1/Sousou no Frieren S01E12.mkv");

        rename(&bangumi_info, src_path, dst_folder).unwrap();

        let content = std::fs::read_to_string(dst_path).unwrap();
        assert_eq!(content, "test");
    }
}
