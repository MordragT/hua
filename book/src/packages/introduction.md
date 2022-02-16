# Packages

In Hua packages describe the installed recipes.

---

# Old

We differentiate in hua between mutable and immutable packages.
In the former the user can change some content of a package that is 
specified in its `package.roc` file. Because by default the user is expected
to change configuration files and such, the installation of a pacakge will
usually create a mutable package. When a package does not allow changing any content,
it is always installed as immutable package.

## Package Types

TODO:
Check if it is possible to let a link search its libraries and stuff in the links working
directory and not in the working directory of the original binary. This would allow us to
make the mutable package dependent on the immutable package and just copying the as mutable
declared files instead of linking them as we do with all the others. This may also be possible
with an overlay, but this seems like a solution that is not very cross platform.

#### Immutable Package

An immutable package is created under the packages name folder  e.g. `/hua/store/firefox/`
and is specified by its version number an a unique hash: `/hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310`
The user is not able to change any files even if they are declared as mutable but the package can be downloaded
if the store is used as cache and the user can just copy the package to another computer.

#### Mutable Package

A mutable package is also created under the packages name folder and is in addition to the version and the hash
furhter specified with the owner: `/hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310-tom`.
This package cannot be used in a cache server but as hua knows which files may potentially be altered, an installation
of the same version again (or resetting of the configuration) can happen without downloading the whole package again.
At the moment a mutable package cannot be converted into an immutable package although this might
be possible in the future.