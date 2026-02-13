export const C = {
    SRC_FILE: 'example.fax',
    TARGET_LANG: 'rs',
    OUT_FMT: 'native',
    OPT_LVL: 2,
    MAX_ROOTS: 128,
    TMP_PREF: '.temp_',
    LLVM_EXT: '.ll',
    EXE_EXT: '',
    TIMEOUT: 300000,
} as const;

export const CMD = {
    LEXER: './faxc/packages/lexer/target/release/lexer "{0}"',
    PARSER: './faxc/packages/parser/zig-out/bin/parser "{0}"',
    SEMA: './faxc/packages/sema/bin/sema "{0}"',
    OPT: './faxc/packages/optimizer/target/release/fax-opt "{0}" --opt-level={level}',
    CODEGEN: './faxc/packages/codegen/build/faxc_cpp "{0}"',
} as const;

export const OPT = {
    NONE: 'none',
    BASIC: 'basic',
    INT: 'intermediate',
    ADV: 'advanced',
    AGGR: 'aggressive',
} as const;
