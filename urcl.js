/*
Language: URCL
Author: funnsam
Description: URCL Highlighting
*/

// hljs.registerLanguage("urcl", function() {
export function get_urcl() {
    const KEYWORDS = [
    	"add", "rsh", "lod", "str", "bge", "nor", "sub", "jmp", "mov", "nop", "imm", "lsh", "inc", "dec",
        "neg", "and", "or" , "not","xnor", "xor","nand", "brl", "brg", "bre", "bne", "bod", "bev", "ble",
        "brz", "bnz", "brn", "brp", "psh", "pop", "cal", "ret", "hlt", "cpy", "brc", "bnc", "mlt", "div",
        "mod", "bsr", "bsl", "srs", "bss", "out", "in" , "dw" ,
        "sete" , "setne", "setg" , "setl" , "setge", "setle", "setc" , "setnc", "llod" , "lstr" , "sdiv",
        "sbrl" , "sbrg" , "sble" , "ssetl", "ssetg", "ssetle"        , "ssetge"
    ];
    return {
        name: "URCL",
        case_insensitive: true,
        contains: [
            hljs.C_LINE_COMMENT_MODE,
            hljs.C_BLOCK_COMMENT_MODE,
            {
                scope: "number",
                begin: "(\\+|\\-)?(0[xX][A-Fa-f0-9]+|0[bB][0-1]+|0[oO][0-7]+|[0-9]+)"
            },
            { scope: "meta"     , begin: "@[\\S]+", end: "[\\s]"      	},
            { scope: "meta"     , begin: "(minreg|minheap|bits)"        },
            { scope: "built_in" , begin: "((\\$|\\#|r|m)[0-9]+|pc|sp)"  },
            { scope: "symbol"   , begin: "\\.[\\S]+"                    },
            { scope: "literal"  , begin: "%[\\S]+" 		                },
            {
                scope: "string",
                variants: [
                    {
                        begin: "\"", end: "\"", illegal: "\\n",
                        contains: [ hljs.BACKSLASH_ESCAPE ]
                    },
                    {
                        begin: "'", end: "'", illegal: "\\n",
                        contains: [ hljs.BACKSLASH_ESCAPE ]
                    }
                ],
            },
            {
                scope: "keyword",
                begin: "(" + KEYWORDS.join("|") + ")\\s",
            },
            {
                scope: "name",
                begin: "[A-Za-z0-9_]+",
            },
        ]
    }
}
