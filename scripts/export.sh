
## export container
#docker export final_project-nominatim-1 | zstd > nominatim-container-us-northeast.zst
## import container
#unzstd nominatim-container-us-northeast.zst
#docker import nominatim-container-us-northeast

# export volume
docker-volume-snapshot create nominatim-data nominatim-data-us-northeast.tar
sudo tar -I"zstd -T0" -cvpf nominatim-data-us-northeast.tar.zst nominatim-data-us-northeast.tar
# extract volume
tar --zstd -xvf nominatim-data-us-northeast.tar.zst
#import volume
docker-volume-snapshot restore nominatim-data-us-northeast.tar nominatim-data


# export /data
#sudo tar -cvzf data-us-northeast.tar.gz /
sudo tar -I zstd -cvpf data-us-northeast.tar.zst /data
# import /data
mkdir /data
#tar -xvf data-us-northeast.tar.gz -C /
tar --zstd -xvf data-us-northeast.tar.zst -C /
