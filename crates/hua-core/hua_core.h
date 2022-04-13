#ifndef hua_core
#define hua_core

#include <stddef.h>
#include <stdint.h>

typedef struct {
  char *name;
  char *version_req;
  char **blobs;
  size_t blobs_len;
} CRequirement;

typedef struct {
  char *left;
  char *right;
} CStringTuple;

typedef struct {
  char *name;
  char *version;
  char *desc;
  uint8_t archs;
  uint8_t platforms;
  char *source;
  char **licenses;
  size_t licenses_len;
  CRequirement *requires;
  size_t requires_len;
  CRequirement *requires_build;
  size_t requires_build_len;
  char *target_dir;
  CStringTuple *envs;
  size_t envs_len;
  char *script;
} CRecipe;

void build_recipe(CRecipe recipe);

#endif