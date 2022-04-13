#include "hua_core.h"
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>

typedef void (*build_recipe_fn)(CRecipe);

void build_recipe(CRecipe recipe) {
  void *handle =
      dlopen("/home/tom/Desktop/hua/target/debug/libhua_core.so", RTLD_LAZY);
  if (!handle) {
    fprintf(stderr, "%s\n", dlerror());
    exit(EXIT_FAILURE);
  }
  build_recipe_fn fn = dlsym(handle, "build_recipe");
  fn(recipe);
}