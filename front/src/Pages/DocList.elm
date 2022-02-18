module Pages.DocList exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Http
import Json.Decode as Decode
import Route


type alias Model =
    { list : List String }


type Msg
    = GotDocList (Result Http.Error DocList)


init : ( Model, Cmd msg )
init =
    ( Model [ "foo", "bar", "baz" ], Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotDocList (Ok newList) ->
            ( { model | list = newList }, Cmd.none )

        _ ->
            ( model, Cmd.none )


view : Model -> Html Msg
view model =
    let
        docLink doc =
            li [] [ a [ href <| Route.documents ++ "/" ++ doc ] [ text doc ] ]
    in
    ul [] <| List.map docLink model.list


type alias DocList =
    List String


getDocList : Cmd Msg
getDocList =
    Http.get
        { url = "/api/documents"
        , expect = Http.expectJson GotDocList docListDecoder
        }


docListDecoder : Decode.Decoder DocList
docListDecoder =
    Decode.field "data" <| Decode.list Decode.string
