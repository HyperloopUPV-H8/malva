# :monkey_face:malva:monkey_face:
malva is a deb package that installs and groups all the dependencies needed for stm32 development on VSCode

### Minimum version Ubuntu 20.04

### Dependencies
```
sudo apt install git curl stlink-tools
```
### How to install
```
curl -s https://api.github.com/repos/HyperloopUPV-H8/malva/releases/latest \
| grep "browser_download_url.*deb" \
| cut -d : -f 2,3 \
| tr -d \" \
| wget -qi -
sudo dpkg -i <malva version>
```
