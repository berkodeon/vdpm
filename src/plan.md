1. publish a registry crate, which will keep plugins.toml file in sync with the plugins directory
2. add github workflows job to compare recent sha with stored sha, if they are different, then update the plugins.toml file and publish new version of the crate
3. use the crate for gathering metadata about plugins, and install through vdpm

- latest sha which changed a specific file:
https://api.github.com/repos/saulpw/visidata/commits?path=visidata/loaders/s3.py&per_page=1

- ls a specific directory
https://api.github.com/repos/saulpw/visidata/contents/visidata/loaders

- we are also able to query if a path contains any file changes after a specific sha. (ask to gpt)


They already tried out to index and organize plugins:
https://github.com/visidata/dlc

Several vd plugins repos:

https://jsvine.github.io/intro-to-visidata/advanced/extending-visidata/

https://github.com/jsvine/visidata-plugins
https://github.com/ajkerrigan/visidata-plugins
https://github.com/anjakefala/vd-plugins

main one (loaders):
https://github.com/saulpw/visidata/tree/develop/visidata/loaders
