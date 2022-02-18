module Route exposing (..)

{- Keeping this in a separate module (although it's against Elm recommendations) because
   route names might overlap with page names, and there's no way to namespace type
   variants contained in the same module. Note also that while there is likely overlap,
   the mapping is not necessarily 1:1. Multiple routes might map to the same page, just
   with different parameters. At the very least, there will be a NotFound page variant
   which by definition doesn't correspond to any route.
-}

import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, s)


type Route
    = DocList
    | DocDetail String
    | SpeakerList
    | SpeakerDetail String
    | SearchResults String


documents : String
documents =
    "documents"


speakers : String
speakers =
    "speakers"


search : String
search =
    "search"


urlParser : Parser (Route -> a) a
urlParser =
    Parser.oneOf
        [ Parser.map DocList (s documents)
        , Parser.map DocDetail (s documents </> Parser.string)
        , Parser.map SpeakerList (s speakers)
        , Parser.map SpeakerDetail (s speakers </> Parser.string)
        , Parser.map SearchResults (s search </> Parser.string)
        ]


urlParse : Url -> Maybe Route
urlParse url =
    Parser.parse urlParser url
