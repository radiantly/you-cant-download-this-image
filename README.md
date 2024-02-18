# You can't download this image

![Site status](https://github.com/radiantly/you-cant-download-this-image/actions/workflows/site.yml/badge.svg)

Or can you? Visit https://youcantdownloadthisimage.com/ to give it a go!

## Running your own server

If you'd just like to test the code that keeps the connection open, run the following

```sh
# Build and start serving lisa.jpg on http://localhost:3000/
make
./serve
```

<details>
<summary>Full setup</summary>

The following assumes that you have [caddy](https://caddyserver.com/) installed with systemd.

```sh
cd /opt                                                                # Navigate to /opt
git clone https://github.com/radiantly/you-cant-download-this-image    # Clone repository
chown -R :caddy you-cant-download-this-image/                          # Set dir group to caddy
cd you-cant-download-this-image && make                                # Build
```

Start and enable the systemd unit file to keep it running:

```sh
ln -s /opt/you-cant-download-this-image/lisa.service /etc/systemd/system/lisa.service
systemctl daemon-reload    # Reload service files on disk
systemctl start lisa       # Start
systemctl enable lisa      # Autostart on boot
```

Configure caddy:

```sh
mv /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak    # backup existing Caddyfile
ln -s /opt/you-cant-download-this-image/Caddyfile /etc/caddy/Caddyfile
systemctl restart caddy
```

</details>

## License

MIT
