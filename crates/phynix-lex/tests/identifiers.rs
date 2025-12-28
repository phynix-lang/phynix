mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_core::token::TokenKind;

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
        "abstract and array as break callable case catch class clone const ",
        "continue declare default die do echo else elseif empty enddeclare ",
        "endfor endforeach endif endswitch endwhile enum eval exit extends ",
        "final finally fn for foreach from function global goto if implements ",
        "include include_once instanceof insteadof interface isset list match ",
        "namespace new or parent print private protected public readonly ",
        "require require_once return self static switch throw trait try unset ",
        "use var while xor yield",
    );

    let k = kinds_php_prefixed(src);

    assert_kinds_eq(
        &k,
        &[
            TokenKind::KwAbstract,
            TokenKind::KwAnd,
            TokenKind::KwArray,
            TokenKind::KwAs,
            TokenKind::KwBreak,
            TokenKind::KwCallable,
            TokenKind::KwCase,
            TokenKind::KwCatch,
            TokenKind::KwClass,
            TokenKind::KwClone,
            TokenKind::KwConst,
            TokenKind::KwContinue,
            TokenKind::KwDeclare,
            TokenKind::KwDefault,
            TokenKind::KwDie,
            TokenKind::KwDo,
            TokenKind::KwEcho,
            TokenKind::KwElse,
            TokenKind::KwElseIf,
            TokenKind::KwEmpty,
            TokenKind::KwEndDeclare,
            TokenKind::KwEndFor,
            TokenKind::KwEndForeach,
            TokenKind::KwEndIf,
            TokenKind::KwEndSwitch,
            TokenKind::KwEndWhile,
            TokenKind::KwEnum,
            TokenKind::KwEval,
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
            TokenKind::KwIf,
            TokenKind::KwImplements,
            TokenKind::KwInclude,
            TokenKind::KwIncludeOnce,
            TokenKind::KwInstanceof,
            TokenKind::KwInsteadof,
            TokenKind::KwInterface,
            TokenKind::KwIsset,
            TokenKind::KwList,
            TokenKind::KwMatch,
            TokenKind::KwNamespace,
            TokenKind::KwNew,
            TokenKind::KwOr,
            TokenKind::KwParent,
            TokenKind::KwPrint,
            TokenKind::KwPrivate,
            TokenKind::KwProtected,
            TokenKind::KwPublic,
            TokenKind::KwReadonly,
            TokenKind::KwRequire,
            TokenKind::KwRequireOnce,
            TokenKind::KwReturn,
            TokenKind::KwSelf,
            TokenKind::KwStatic,
            TokenKind::KwSwitch,
            TokenKind::KwThrow,
            TokenKind::KwTrait,
            TokenKind::KwTry,
            TokenKind::KwUnset,
            TokenKind::KwUse,
            TokenKind::KwVar,
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
