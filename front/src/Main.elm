module Main exposing (main)

import Browser
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Http
import Json.Decode as D
import Url



-- MAIN


main : Program () Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlChange = UrlChanged
        , onUrlRequest = LinkClicked
        }



-- MODEL


type alias Model =
    { key : Nav.Key
    , url : Url.Url
    , page : Page
    }


type Page
    = DocList
    | DocDetail



-- | SpeakerList
-- | SpeakerDetail
-- | SearchResults


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    ( { key = key, url = url, page = DocList }, Cmd.none )



-- UPDATE


type Msg
    = LinkClicked Browser.UrlRequest
    | UrlChanged Url.Url
    | GotDocList (Result Http.Error DocList)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LinkClicked urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( model, Nav.pushUrl model.key (Url.toString url) )

                Browser.External href ->
                    ( model, Nav.load href )

        UrlChanged url ->
            ( { model | url = url }, getDocList )

        GotDocList _ ->
            ( model, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title = "Quetzal: Databáze sond mluvené češtiny"
    , body =
        [ text "Právě jste na adrese:"
        , b [] [ text (Url.toString model.url) ]
        , ul []
            [ viewLink "/documents"
            , viewLink "/speakers"
            , viewLink "/search"
            ]
        ]
    }


viewLink : String -> Html msg
viewLink path =
    li [] [ a [ href path ] [ text path ] ]



-- HTTP


type alias Doc =
    { id : String }


type alias DocList =
    List Doc


getDocList : Cmd Msg
getDocList =
    Http.get
        { url = "/api/documents"
        , expect = Http.expectJson GotDocList docListDecoder
        }


docDecoder : D.Decoder Doc
docDecoder =
    D.map Doc <| D.field "id" D.string


docListDecoder : D.Decoder DocList
docListDecoder =
    D.field "data" <| D.list docDecoder
