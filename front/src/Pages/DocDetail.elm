module Pages.DocDetail exposing (..)

import Html exposing (..)
import Json.Decode as Decode


type alias Model =
    { docId : String }


type Msg
    = Msg


init : String -> ( Model, Cmd msg )
init docId =
    ( Model docId, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update _ model =
    ( model, Cmd.none )


view : Model -> Html Msg
view model =
    text model.docId


type alias Doc =
    { id : String }


docDecoder : Decode.Decoder Doc
docDecoder =
    Decode.map Doc <| Decode.field "id" Decode.string
