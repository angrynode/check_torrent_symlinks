# check_torrent_download

This is a helper script for torrentmanager. In torrentmanager, all torrents are stored in a single downloads folder, each torrent in a subdirectory named after it's torrent ID. Then, symlinks are created in a special user-accessible folder to those torrents, so that user-visible content can be renamed/removed without affecting torrent seeding.

This script makes sure all torrents in QBittorrent are set to the correct download location, and fixes it otherwise. Then, it recursively walks through the symlinks folder making sure all the symlinks in there actually point to the correct location: it does so by checking if the symlink destination contains a torrent ID, and if so, making sure it matches the actual (new) torrent location.

# Usage

```
$ check_torrent_download TORRENTS_DIR SYMLINKS_DIR
```

# License

GNU aGPL v3.0



check_torrent_download TORRENT_DIR SYMLINK_DIR
