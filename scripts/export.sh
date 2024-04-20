
# export container
docker export final_project-nominatim-1 | gzip > nominatim-container-us-northeast.gz
docker-volume-snapshot create nominatim-data nominatim-data-us-northeast.tar
sudo tar -I"zstd -T0" -cvpf nominatim-data-us-northeast.tar.zst nominatim-data-us-northeast.tar

# export /data
#sudo tar -cvzf data-us-northeast.tar.gz /data
sudo tar -I zstd -cvpf data-us-northeast.tar.zst /data


# import nominatim data
docker import final_project-nominatim-1
docker-volume-snapshot restore nominatim-data-us-northeast.tar nominatim-data

# import /data
mkdir /data
tar -xvf data-us-northeast.tar.gz -C /data
tar --zstd -xvf data-us-northeast.tar.zst /data
