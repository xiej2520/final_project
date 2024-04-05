# download test_data
wget https://github.com/maptiler/tileserver-gl/releases/download/v1.3.0/test_data.zip
unzip test_data.zip -d tileserver-gl
cp static/tileserver-config.json tileserver-gl/config.json
cd tileserver-gl

docker run --rm -it -v $(pwd):/data -p 8080:8080 maptiler/tileserver-gl
