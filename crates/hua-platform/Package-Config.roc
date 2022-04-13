platform "hua"
    requires {} { recipe : Recipe }
    exposes [ Recipe, Requirement, Tuple ]
    packages {}
    imports []
    provides [ mainForHost ]

Requirement : 
    { name : Str
    , versionReq : Str
    , blobs : List Str
    }

Tuple left right : { left : left, right : right }

Recipe : 
    { name : Str
    , version : Str
    , desc : Str
    , archs : U8
    , platforms : U8
    , source : Str
    , licenses : List Str
    , requires : List Requirement
    , requiresBuild : List Requirement
    , targetDir : Str
    , envs : List (Tuple Str Str)
    , script : Str
    }

mainForHost : Recipe
mainForHost = recipe
