# sfss
Simple FileSharing Service

This is a tiny, >5MBs(release mode), filesharing service.

## Features
This run a webserver with a simple file upload, and then saves the uploads for easy sharing. Much like pastebin for files.

/upload for the web interface.
/upload/api is not yet functional.

## Behind the scenes
The webserver recives the file on the /upload and endpoints and saves it to a temporary file.
The webserver then hashes the file using the xxhash3 algorithm and saves the file in /var/www/uploads with the hash encoded as base62(0-9a-zA-Z) as the filename.

## Install
Populate the .env/docker-compose file with the proper environment variables  
`SFSS_TITLE` is the title for the website.  
`SFSS_LABEL` is the label for the file select button.  
`SFSS_ROOT` this is used for if the website isnt hosted at the root of the domain, example `https://example.com/share/`, then this would be `/share`  
`SFSS_URL` this is the url that the site is hosted on, in the above example this would be `https://example.com`  
`SFSS_LOCATION` this is the location for storing the files, if run in docker this should be `/var/sfss`  

Either build the webserver with cargo, `cargo build --release` or use docker, `docker-compose up -d`