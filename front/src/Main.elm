module Main exposing (main)

import Browser exposing (Document)
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Lazy exposing (lazy)
import Pages.DocDetail as DocDetail
import Pages.DocList as DocList
import Route
import Url exposing (Url)



----------------------------------------------------------------------------------- Main {{{1


main : Program () Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = \_ -> Sub.none
        , onUrlChange = ChangedUrl
        , onUrlRequest = ClickedLink
        }



---------------------------------------------------------------------------------- Model {{{1


type alias Model =
    { page : Page
    , key : Nav.Key
    }


type Page
    = DocList DocList.Model
    | DocDetail DocDetail.Model
    | NotFound


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    updatePage url { page = NotFound, key = key }



-------------------------------------------------------------------------------- Updates {{{1


type Msg
    = ClickedLink Browser.UrlRequest
    | ChangedUrl Url.Url
    | GotDocListMsg DocList.Msg
    | GotDocDetailMsg DocDetail.Msg


update : Msg -> Model -> ( Model, Cmd Msg )
update mainMsg mainModel =
    case mainMsg of
        ClickedLink urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( mainModel, Nav.pushUrl mainModel.key (Url.toString url) )

                Browser.External href ->
                    ( mainModel, Nav.load href )

        ChangedUrl url ->
            updatePage url mainModel

        GotDocListMsg msg ->
            case mainModel.page of
                DocList model ->
                    toDocList mainModel (DocList.update msg model)

                _ ->
                    ( mainModel, Cmd.none )

        GotDocDetailMsg msg ->
            case mainModel.page of
                DocDetail model ->
                    toDocDetail mainModel (DocDetail.update msg model)

                _ ->
                    ( mainModel, Cmd.none )


updatePage : Url -> Model -> ( Model, Cmd Msg )
updatePage url model =
    case Route.urlParse url of
        Just Route.DocList ->
            toDocList model <| DocList.init

        Just (Route.DocDetail docId) ->
            toDocDetail model <| DocDetail.init docId

        _ ->
            ( { model | page = NotFound }, Cmd.none )


toDocList : Model -> ( DocList.Model, Cmd DocList.Msg ) -> ( Model, Cmd Msg )
toDocList model ( docList, cmd ) =
    ( { model | page = DocList docList }, Cmd.map GotDocListMsg cmd )


toDocDetail : Model -> ( DocDetail.Model, Cmd DocDetail.Msg ) -> ( Model, Cmd Msg )
toDocDetail model ( docDetail, cmd ) =
    ( { model | page = DocDetail docDetail }, Cmd.map GotDocDetailMsg cmd )



----------------------------------------------------------------------------------- View {{{1


view : Model -> Document Msg
view mainModel =
    let
        content =
            case mainModel.page of
                DocList model ->
                    DocList.view model |> Html.map GotDocListMsg

                DocDetail model ->
                    DocDetail.view model |> Html.map GotDocDetailMsg

                NotFound ->
                    text "Požadovaná stránka nebyla nalezena."
    in
    { title = "Quetzal: Databáze sond mluvené češtiny"
    , body =
        [ lazy viewHeader mainModel.page
        , content
        , viewFooter
        ]
    }


viewHeader : Page -> Html Msg
viewHeader page =
    let
        logo =
            h1 [] [ text "Quetzal" ]

        links =
            ul []
                [ navLink { url = Route.documents, caption = "Sondy" }
                , navLink { url = Route.speakers, caption = "Mluvčí" }
                , navLink { url = Route.search, caption = "Vyhledávání" }
                ]

        navLink : { url : String, caption : String } -> Html msg
        navLink { url, caption } =
            li [] [ a [ href url ] [ text caption ] ]
    in
    nav [] [ logo, links ]


viewFooter : Html Msg
viewFooter =
    footer [] [ text "&copy; ÚČNK 2022" ]



-- vi: foldmethod=marker
