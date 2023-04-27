# 更新说明

## 读写分离
### configuration.rs
添加 read_fd 和 write_fd 到  Configuration 结构中
其中 read_fd  和 write_fd 相同时 和旧有代码兼容
注意：在使用读写分离时,read_fd与raw_fd相同


### platform/ios/device.rs

#### Queue结构更改
添加 read_tun write_tun
在Read 和 Write 时 若 read_tun == write_tun 则 与旧代码兼容

#### Device

#####  new 初始化更改
若Configuration中存在read_fd 则使用读写分离
否则读写分离的tun都是一个

##### Read Write traits 更改

使用 Queue的Read 和 Write 而不是向原来一样直接使用queue中间的tun来Read Write



#### 也许可以直接修改Fd