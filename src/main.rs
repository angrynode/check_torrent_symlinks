use camino::Utf8PathBuf;
use hightorrent::Torrent;
use hightorrent_api::{Api, QBittorrentClient};

use std::borrow::Borrow;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // check_torrent_symlinks DOWNLOADDIR SYMLINKDIR
    //
    // Ensure every torrent from qbittorrent is in DOWNLOADDIR/HASH.
    // Then make sure all symlinks in SYMLINKDIR (recursive) point
    // to that directory.

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let mut expected_dir: String = args.next().unwrap();
    if expected_dir.ends_with("/") {
        expected_dir.pop();
    }

    let mut symlink_dir = args.next().unwrap();
    if symlink_dir.ends_with("/") {
        symlink_dir.pop();
    }

    let api = QBittorrentClient::login("http://localhost:8080", "admin", "adminadmin")
        .await
        .unwrap();
    let list = api.list().await.unwrap().to_vec();

    let weird_torrents: Vec<Torrent> = list
        .iter()
        .filter_map(|torrent| {
            if !torrent.path.starts_with(&expected_dir) {
                Some(torrent.clone())
            } else {
                None
            }
        })
        .collect();

    for torrent in &weird_torrents {
        let new_dir = format!("{}/{}", expected_dir, torrent.id);
        println!("[{}] Setting location to {new_dir}", torrent.name);
        api.set_location(&torrent.hash.borrow().into(), &new_dir)
            .await
            .unwrap();
    }

    println!(
        "\n\nFound {}/{} weird torrents and fixed them",
        weird_torrents.len(),
        list.len()
    );

    let torrents_dir = Utf8PathBuf::from(expected_dir);
    let symlink_dir = Utf8PathBuf::from(symlink_dir);
    let hashes: Vec<String> = list.iter().map(|torrent| torrent.id.to_string()).collect();

    let mut fixed_symlinks: Vec<Utf8PathBuf> = vec![];
    let mut broken_symlinks: Vec<(Utf8PathBuf, Utf8PathBuf)> = vec![];

    fix_content_path(
        symlink_dir,
        torrents_dir,
        &hashes,
        &mut fixed_symlinks,
        &mut broken_symlinks,
    );
    println!("\nStill broken:");
    for (orig, _dest) in &broken_symlinks {
        println!("- {orig}");
    }
    println!("\n\nFixed {} symlinks.", fixed_symlinks.len());
    println!("\n\nStill broken {} symlinks.", broken_symlinks.len());
}

pub fn fix_content_path(
    symlink_dir: Utf8PathBuf,
    torrents_dir: Utf8PathBuf,
    hashes: &Vec<String>,
    fixed_symlinks: &mut Vec<Utf8PathBuf>,
    broken_symlinks: &mut Vec<(Utf8PathBuf, Utf8PathBuf)>,
) {
    for entry in symlink_dir.read_dir_utf8().unwrap() {
        let entry = entry.unwrap();
        let entry = entry.path();
        if entry.is_dir() {
            fix_content_path(
                entry.to_path_buf(),
                torrents_dir.clone(),
                hashes,
                fixed_symlinks,
                broken_symlinks,
            );
        } else if entry.is_symlink() {
            let link = entry.read_link_utf8().unwrap();
            if !link.starts_with(&torrents_dir) {
                // Find hash in existing link
                let potential_hash = link
                    .components()
                    .find(|comp| hashes.contains(&comp.to_string()));
                let Some(hash) = potential_hash else {
                    println!("{} symlink broken in file {}", link, entry);
                    broken_symlinks.push((entry.to_path_buf(), link.to_path_buf()));
                    continue;
                };

                // Find the parts after the hash
                let mut final_parts = Utf8PathBuf::new();
                let mut found_hash = false;
                for part in link.components() {
                    if hash == part {
                        found_hash = true;
                        continue;
                    }

                    if found_hash {
                        final_parts.push(part);
                    }
                }

                let new_link = torrents_dir.join(&hash).join(final_parts);
                fixed_symlinks.push(entry.to_path_buf());
                println!("{}->{}", link, new_link);
                std::fs::remove_file(entry).unwrap();
                std::os::unix::fs::symlink(new_link, entry).unwrap();
            }
        }
    }
}
