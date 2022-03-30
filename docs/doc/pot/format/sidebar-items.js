initSidebarItems({"enum":[["Kind","The type of an atom."],["Nucleus","A value contained within an [`Atom`]."],["Special","A special value type."]],"fn":[["read_atom","Reads an atom."],["read_atom_header","Reads an atom header (kind and argument)."],["read_header","Reads a Pot header. See `write_header` for more information. Returns the version number contained within."],["write_atom_header","Writes an atom header into `writer`."],["write_bool","Writes a [`Kind::Special`] atom with either [`Special::True`] or [`Special::False`]."],["write_bytes","Writes an [`Kind::Bytes`] atom with the given value."],["write_f32","Writes an [`Kind::Float`] atom with the given value."],["write_f64","Writes an [`Kind::Float`] atom with the given value."],["write_header","Writes the Pot header. A u32 written in big endian. The first three bytes are ‘Pot’ (`0x506F74`), and the fourth byte is the version. The first version of Pot is 0."],["write_i128","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i16","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i24","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i32","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i48","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i64","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_i8","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_named","Writes a [`Kind::Special`] atom with [`Special::Named`]."],["write_none","Writes a [`Kind::Special`] atom with [`Special::None`]."],["write_special","Writes a [`Kind::Special`] atom."],["write_str","Writes an [`Kind::Bytes`] atom with the bytes of the string."],["write_u128","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u16","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u24","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u32","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u48","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u64","Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible."],["write_u8","Writes an [`Kind::UInt`] atom with the given value."],["write_unit","Writes a [`Kind::Special`] atom with [`Special::Unit`]."]],"struct":[["Atom","An encoded [`Kind`], argument, and optional contained value."],["Float","A floating point number that can safely convert between other number types using compile-time evaluation when possible."],["Integer","An integer type that can safely convert between other number types using compile-time evaluation."]]});