
## export container
#docker export final_project-nominatim-1 | zstd > nominatim-container-us-northeast.zst
## import container
#unzstd nominatim-container-us-northeast.zst
#docker import nominatim-container-us-northeast


### NOMINATIM VOLUME
# export nominatim volume
docker-volume-snapshot create nominatim-data nominatim-data-us-northeast.tar
sudo tar -I"zstd -T0" -cvpf nominatim-data-us-northeast.tar.zst nominatim-data-us-northeast.tar
# extract nominatim volume
tar --zstd -xvf nominatim-data-us-northeast.tar.zst
# import nominatim volume
docker-volume-snapshot restore nominatim-data-us-northeast.tar nominatim-data

### FULL /data directory
# export /data
#sudo tar -cvzf data-us-northeast.tar.gz /
sudo tar -I zstd -cvpf data-us-northeast.tar.zst /data
# import /data
mkdir /data
#tar -xvf data-us-northeast.tar.gz -C /
tar --zstd -xvf data-us-northeast.tar.zst -C /

###### PARTS ######
### OSRM in /data
sudo tar -I zstd -cvpf data-us-northeast-osrm.tar.zst /data/osrm
# extract
tar --zstd -xvf data-us-northesat-osrm.tar.zst -C /

### Tileserver and tiles
sudo tar -I zstd -cvpf data-us-northeast-tiles-tileserver.tar.zst /data/tiles /data/tileserver
tar --zstd -xvf data-us-northeast-tiles-tileserver.tar.zst -C /

### PHOTON in /data
sudo tar -I zstd -cvpf data-us-northeast-photon.tar.zst /data/photon
tar --zstd -xvf data-us-northeast-photon.tar.zst -C /

#### NOMINATIM NEEDS THE .osm.pbf FILE
