# rgrcat (WIP) 
An alternative to [grcat](https://github.com/garabik/grc/blob/master/grcat) (belong to [grc](https://github.com/garabik/grc)) written in Rust.

This project is work in progress, the coloring result is not guaranteed to be correct.


## Build and run
Use `cargo install --path .` to install it. Don't forget to add `~/.cargo/bin` to your $PATH.

The usage of rgrcat is the same as grcat. The usage is: `rgrcat conffile` (Currently need to install grc to obtain conffile). For example, `systemctl status mariadb.service | rgrcat conf.systemctl`.
