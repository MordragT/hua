app "automake"
    packages { pf: "../crates/hua-platform" }
    imports []# [ pf.Recipe ]
    provides [ recipe ] to pf

recipe =
    name = "automake"
    version = "1.16.4"

    { name
    , version
    , desc : "A GNU tool for automatically creating Makefiles"
    , archs : 1
    , platforms : 1
    , source : "https://ftp.gnu.org/gnu/automake/automake-\(version).tar.gz"
    , licenses : [ "GPLv2" ]
    , requires : []
    , requiresBuild : []
    , targetDir : "\(name)-\(version)"
    , envs : []
    , script : "echo Nothing todo"
    }