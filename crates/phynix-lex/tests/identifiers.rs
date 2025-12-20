mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_lex::TokenKind;

#[test]
fn idents_and_varidents() {
    let k = kinds_php_prefixed("$über über _x");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn namespaces_and_sigils() {
    let k = kinds_php_prefixed("$$foo ${bar} Foo\\Bar");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Dollar,
            TokenKind::VarIdent,
            TokenKind::Dollar,
            TokenKind::LBrace,
            TokenKind::Ident,
            TokenKind::RBrace,
            TokenKind::Ident,
            TokenKind::Backslash,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn all_keywords_are_recognized() {
    let src = concat!(
        "abstract break catch class clone const continue declare default die ",
        "do elseif enddeclare endfor endforeach endif endswitch endwhile enum ",
        "exit extends final finally fn for foreach from function global goto ",
        "implements include include_once instanceof interface match namespace ",
        "new or private protected public readonly require require_once return ",
        "static switch throw trait try use while xor yield",
    );

    let k = kinds_php_prefixed(src);

    assert_kinds_eq(
        &k,
        &[
            TokenKind::KwAbstract,
            TokenKind::KwBreak,
            TokenKind::KwCatch,
            TokenKind::KwClass,
            TokenKind::KwClone,
            TokenKind::KwConst,
            TokenKind::KwContinue,
            TokenKind::KwDeclare,
            TokenKind::KwDefault,
            TokenKind::KwDie,
            TokenKind::KwDo,
            TokenKind::KwElseIf,
            TokenKind::KwEndDeclare,
            TokenKind::KwEndFor,
            TokenKind::KwEndForeach,
            TokenKind::KwEndIf,
            TokenKind::KwEndSwitch,
            TokenKind::KwEndWhile,
            TokenKind::KwEnum,
            TokenKind::KwExit,
            TokenKind::KwExtends,
            TokenKind::KwFinal,
            TokenKind::KwFinally,
            TokenKind::KwFn,
            TokenKind::KwFor,
            TokenKind::KwForeach,
            TokenKind::KwFrom,
            TokenKind::KwFunction,
            TokenKind::KwGlobal,
            TokenKind::KwGoto,
            TokenKind::KwImplements,
            TokenKind::KwInclude,
            TokenKind::KwIncludeOnce,
            TokenKind::KwInstanceof,
            TokenKind::KwInterface,
            TokenKind::KwMatch,
            TokenKind::KwNamespace,
            TokenKind::KwNew,
            TokenKind::KwOr,
            TokenKind::KwPrivate,
            TokenKind::KwProtected,
            TokenKind::KwPublic,
            TokenKind::KwReadonly,
            TokenKind::KwRequire,
            TokenKind::KwRequireOnce,
            TokenKind::KwReturn,
            TokenKind::KwStatic,
            TokenKind::KwSwitch,
            TokenKind::KwThrow,
            TokenKind::KwTrait,
            TokenKind::KwTry,
            TokenKind::KwUse,
            TokenKind::KwWhile,
            TokenKind::KwXor,
            TokenKind::KwYield,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn non_keyword_ascii_idents_fall_back_to_ident_for_each_length_bucket() {
    let src = concat!(
        "aaaaaa ",
        "aaaaaaa ",
        "aaaaaaaa ",
        "aaaaaaaaa ",
        "aaaaaaaaaa ",
        "aaaaaaaaaaaa",
    );

    let k = kinds_php_prefixed(src);
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}
