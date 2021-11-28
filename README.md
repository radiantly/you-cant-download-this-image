# You can't download this image

Or can you? Visit https://youcantdownloadthisimage.online/ to give it a go!

## Running your own server

If you'd just like to test the code that keeps the connection open, run the following

```sh
# Install dependencies and start serving lisa.jpg on http://localhost:3000/
cargo run
```

<details>
<summary>Full setup</summary>
The following assumes that you have nginx installed on a linux system with systemd.

```sh
# Clone this repository
git clone https://github.com/radiantly/you-cant-download-this-image
cd you-cant-download-this-image

# Release build
cargo build --release
```

Create a systemd unit file to keep it running:

These are the contents of `/etc/systemd/system/lisa.service` _(replace paths as needed)_

```
[Unit]
Description=You can't download this image server
After=network.target

[Service]
Type=simple
WorkingDirectory=/root/you-cant-download-this-image
ExecStart=/root/you-cant-download-this-image/target/release/you-cant-download-this-image
Restart=always

[Install]
WantedBy=multi-user.target
```

```sh
systemctl daemon-reload   # Reload service files on diskfi
systemctl start lisa      # Start
systemctl enable lisa     # Atuostart on boot
```

For the ssl cert, follow the instructions on Certbot's website.

Finally, something like this can be added to the nginx config:

```
server {
    if ($host = www.youcantdownloadthisimage.online) {
        return 301 https://$host$request_uri;
    } # managed by Certbot


    if ($host = youcantdownloadthisimage.online) {
        return 301 https://$host$request_uri;
    } # managed by Certbot

    listen 80;
}

server {
    listen 443 ssl;
    root /root/you-cant-download-this-image/public;
    server_name youcantdownloadthisimage.online www.youcantdownloadthisimage.online;
    ssl_certificate /etc/letsencrypt/live/youcantdownloadthisimage.online/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/youcantdownloadthisimage.online/privkey.pem; # managed by Certbot

    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot

    location / {
        add_header X-Frame-Options "SAMEORIGIN";
        add_header Access-Control-Allow-Origin https://youcantdownloadthisimage.online;
    }

    location /lisa.jpg {
        proxy_pass              http://localhost:3000;
        proxy_redirect          http://localhost:3000 https://youcantdownloadthisimage.online;
    }
}
```

</details>

## License

MIT
