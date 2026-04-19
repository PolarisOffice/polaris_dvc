/*
 * Copyright 2022 Hancom Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#pragma once

#include <vector>
#include <string>
#include <map>

#include "json.h"

#include "../export/ExportInterface.h"

typedef unsigned int    UINT;

#define UNDEFVALUE        9999

typedef struct JsonINT
{
    INT    value = UNDEFVALUE;
    INT    min = UNDEFVALUE;
    INT    max = UNDEFVALUE;
}jsonint;

typedef struct JsonUINT
{
    INT    value = UNDEFVALUE;
    INT    min = UNDEFVALUE;
    INT    max = UNDEFVALUE;
}jsonuint;

enum objectType {
    objectType_boolean = 0,
    objectType_number,
    objectType_string,
};


////////////////////////////////////////////////////////////////////////////////////////////////
// CharShape

enum LangType {
    langBase = 0,
    langHangul,
    langEng,
    langCha,
    langJpn,
    langFor,
    langSimbol,
    langUser
};

enum ShadowType {
    STNone = 0,
    UnContinue,
    Continue
};

enum LinePosition {
    LPNone = 0,
    Top,
    Bottom,
    Left,
    Right
};

enum LineShape {
    LSNone = 0,
    SOLID,
    DOT, 
    DASH,    
    DASH_DOT,
    DASH_DOT_DOT,
    LONGDASH,
    CIRCLE,
    DOUBLESLIM,
    SLIMTHICK,
    THICKSLIM,
    SLIMTHICKSLIM,
    WAVE,
    DOUBLEWAVE,
    THICK3D,
    THICKREV3D,
    THREEDLINE,
    REV3D,
    NITEMS
};

enum PattonType
{
    PTNone = 0,
    STYLE1,
    STYLE2,
    STYLE3,
    STYLE4,
    STYLE5,
    STYLE6
};

struct LangSet {
    LangType                    langType;
    std::vector<std::string>    fontName;
    JsonINT                        rSize;
    JsonINT                        position;
    JsonINT                        ratio;
    JsonINT                        spacing;
    std::vector<UINT>            checkedList;    // 값의 설정 유무
};

struct BorderInfo {
    LinePosition    position;
    LineShape        lineShape;
    float            size;
    UINT            color;
    std::vector<UINT>    checkedList;    // 값의 설정 유무
};


enum DVCType {
    CharShape = 0,
    ParaShape,
    Table,
    SpecialCharacter,
    OutlineShape,
    Bullet,
    ParaNumBullet,
    Style,
    Hyperlink,
    Macro
};

__interface ICheckListObj{
    virtual DVCType                            getType() = 0;
    virtual std::map<std::string, int>        getMapData() = 0;
    virtual std::vector<UINT>        getCheckedList() = 0;
};


__interface ICharShape : public ICheckListObj
{
    virtual JsonUINT            getFontSize();
            
    virtual LangType        getLangType();
    virtual std::vector<std::string> getFontNames();
    virtual JsonUINT        getRSize();
    virtual JsonINT            getPosition();
    virtual JsonUINT        getRatio();
    virtual JsonINT            getSpacing();

    virtual bool            getBold();
    virtual bool            getItalic();
    virtual bool            getUnderline();
    virtual bool            getStrikeout();
    virtual bool            getOoutline();
    virtual bool            getEmboss();
    virtual bool            getEngrave();
    virtual bool            getShadow();
    virtual bool            getSupscript();
    virtual bool            getSubscript();

    virtual ShadowType        getShadowType();
    virtual INT                getShadowXDirection();
    virtual INT                getShadowYDirection();
    virtual UINT            getShadowColor();

    virtual LinePosition    getUnderlinePos();
    virtual LineShape        getUnderlineShape();
    virtual UINT            getUnderlineColor();

    virtual LineShape        getStrikeoutShape();
    virtual UINT            getStrikeColor();

    virtual LineShape        getOutlineShape();
    virtual bool            getEmptyspace();
    virtual UINT            getPoint();
    virtual bool            getKerning();

    virtual BorderInfo*        getBorderInfo(LinePosition pos);
    virtual std::vector<BorderInfo*> getBorderInfos();
    virtual UINT            getBackgroundColor();
    virtual UINT            getPattonColor();
    virtual PattonType        getPattonType();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// ParaShape

enum HAlignType {    //    ["양쪽 정렬", "왼쪽 정렬", "가운데 정렬", "오른쪽 정렬", "배분 정렬", "나눔 정렬"]
    HAlignJustify = 0,
    HAlignLeft,
    HAlignCenter,
    HAlignRight,
    HAlignDistribute,
    HAlignDistributeSpace
};

enum VAlignType {
    VAlignBaseLine = 0,
    VAlignTop,
    VAlignMiddle,
    VAlignBottom
};

enum LineSpacingType {    // ["글자에 따라", "고정값", "여백만 지정", "최소"] },
    LineSpacing_PERCENT = 0,
    LineSpacing_FIXED,
    LineSpacing_BETWEEN_LINES,
    LineSpacing_AT_LEAST
};

enum ParaType {
    ParaNone = 0,
    ParaOutline,
    ParaNumber,
    ParaBullet
};

enum TabType {
    TabLeft = 0,
    TabRight,
    TabMiddle,
    TabPoint
};

enum LineBreakLatinWord {
    KeepWord = 0,        // 단어
    Hyphenation,        // 하이픈
    BreakWord            // 글자
};

enum LineBreakNonLatinWord {
    Non_KeepWord = 0,        // 단어
    Non_BreakWord            // 글자
};

struct TabTypes {
    TabType                        tabType;
    LineShape                    tabShape;
    UINT                        position;
    std::vector<UINT>            checkedList;    // 값의 설정 유무
};

__interface IParaShape : public ICheckListObj
{
    virtual HAlignType                getHorizontal();
    virtual JsonINT                    getMarginLeft();
    virtual JsonINT                    getMarginRight();
    
    virtual bool                    getFirstline();
    virtual JsonUINT                getIndent();
    virtual JsonUINT                getOutdent();
    virtual LineSpacingType            getLinespacing();
    virtual JsonUINT                getLinespacingvalue();
    virtual JsonUINT                getSpacingParaup();
    virtual JsonUINT                getSpacingParabottom();
    virtual bool                    getSpacingGridpaper();
    virtual bool                    getLinebreakKorean();        // TRUE = 글자, FALSE = 어절
    virtual LineBreakLatinWord        getLinebreakEnglish();        // TRUE = 단어, FALSE = 하이픈
    virtual JsonUINT                getLinebreakCondense();
    
    virtual ParaType                getParaType();
    virtual UINT                    getParatypeValue();
    
    virtual bool                    getWidowOrphan();            // "외톨이줄 보호"
    virtual bool                    getKeppWithNext();            // "다음 문단과 함께"
    virtual bool                    getKeepLinesTogether();        // "문단 보호"
    virtual bool                    getPageBreakBefore();        // "문단 앞에서 항상 쪽 나눔"
    virtual bool                    getFontLineHeight();        // "글꼴에 어울리는 줄 높이"
    virtual bool                    getLineWrap();                // "한 줄로 입력"
    virtual bool                    getAutoSpaceEasianEng();    // "한그과 영어 간격을 자동 조절"
    virtual bool                    getAutoSpaceEasianNum();    // "한글과 숫자 간격을 자동 조절"
    virtual VAlignType                getVerticalAlign();
    virtual std::vector<TabTypes*>    getTabTypes(TabType type);
    virtual bool                    getAutotabIntent();
    virtual bool                    getAutotabPararightEnd();
    virtual JsonUINT                getBaseTabSpace();
    
    virtual BorderInfo*                getBorderInfo(LinePosition pos);
    virtual UINT                    getBackgroundColor();
    virtual UINT                    getPattonColor();
    virtual PattonType                getPattonType();

    virtual    bool                    getSpacingLeft();
    virtual    bool                    getSpacingRight();
    virtual    bool                    getSpacingTop();
    virtual    bool                    getSpacingBottom();
    virtual    bool                    getSpacingIgnore();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// Table

enum PosType {
    WRAP_TOP_AND_BOTTOM = 0,
    BRING_IN_FRONT_OF_TEXT,
    SEND_BEHIND_TEXT,
    WRAP_SQUARE
};

enum TextPosType {
    BOTH_SIDES = 0,
    LEFT_ONLY,
    RIGHT_ONLY,
    LARGEST_ONLY
};

enum HorizontalType {
    HPAPER = 0,
    HPAGE,
    HCOLUMN,
    HPARA
};

enum HorizontalDirection {
    HLEFT = 0,
    HCENTER,
    HRIGHT,
    HINSIDE,
    HOUTSIDE
};

enum VerticalType{
    VPAPER = 0,
    VPAGE,
    VCOLUMN,
};

enum VerticalDirection {
    VTOP = 0,
    VCENTER,
    VBOTTOM
};

enum NumType {
    NUM_NONE = 0,
    NUM_PICTURE,
    NUM_TABLE,
    NUM_FORMULA
};

enum CaptionPosType {    
    CapPos_LEFTTOP = 0,
    CapPos_TOP,
    CapPos_RIGHTTOP,
    CapPos_LEFT,
    CapPos_NONE,
    CapPos_RIGHT,
    CapPos_LEFTBOTTOM,
    CapPos_BOTTOM,
    CapPos_RIGHTBOTTOM
};

enum BGFillType {
    BGFILL_NONE = 0,
    BGFILL_COLOR,
    BGFILL_GRADATION
};

enum BGPattonType {    // "없음", "수평선", "수직선", "하향대각선", "상향대각선", "눈금무늬", "체크무늬"
    HATCH_STYLE1 = 0,
    HATCH_STYLE2,
    HATCH_STYLE3,
    HATCH_STYLE4,
    HATCH_STYLE5,
    HATCH_STYLE6,
    HATCH_STYLE7
};

enum BGGradationType {
    LINE = 0,
    RADIAL,
    SQUARE,
    CONIAL
};

enum PicFillType {
    TILE = 0,
    TILE_HORZ_TOP,
    TILE_HORZ_BOTTOM,
    TILE_VERT_LEFT,
    TILE_VERT_RIGHT,
    TOTAL,
    CENTER,
    CENTER_TOP,
    CENTER_BOTTOM,
    LEFT_CENTER,
    LEFT_TOP,
    LEFT_BOTTOM,
    RIGHT_CENTER,
    RIGHT_TOP,
    RIGHT_BOTTOM,
    ZOOM
};

enum EffectType {
    NONE = 0,
    GRAY,
    BLACK,
    ORG
};

__interface ITable : public ICheckListObj
{
    virtual JsonUINT                getSizeWidth();
    virtual JsonUINT                getSizeHeight();
    virtual bool                    getSizeFixed();
    virtual bool                    getTreatAsChar();
    virtual PosType                    getPosition();
    virtual TextPosType                getTextPos();
    virtual HorizontalType            getHType();
    virtual HorizontalDirection        getHDirection();
    virtual JsonINT                    getHValue();
    virtual VerticalType            getVType();
    virtual VerticalDirection        getVDirection();
    virtual JsonINT                    getVValue();
    virtual bool                    getSoflowWithText();
    virtual bool                    getSoalLowOverlap();
    virtual bool                    getSoholdanchrObj();
    virtual bool                    getParallel();
    virtual JsonINT                    getRotation();
    virtual JsonUINT                getGradientH();
    virtual JsonUINT                getGradientV();
    virtual NumType                    getNumverType();
    virtual bool                    getObjProtect();
    virtual JsonUINT                getMarginLeft();
    virtual JsonUINT                getMarginRight();
    virtual JsonUINT                getMarginTop();
    virtual JsonUINT                getMarginBottom();
    virtual CaptionPosType            getCaptionPosType();
    virtual JsonUINT                getCaptionSize();
    virtual JsonUINT                getCaptionSpacing();
    virtual bool                    getCaptionSocapFullSize();
    virtual bool                    getCaptionLineWrap();

    virtual BorderInfo*                getBorderInfo(LinePosition pos);
    virtual std::vector<BorderInfo*> getBorderInfos();

    virtual UINT                    getBorderCellSpacing();
    virtual BGFillType                getBGFillType();
    virtual UINT                    getBGFillFaceColor();
    virtual UINT                    getBGFillPattonColor();
    virtual BGPattonType            getBGFillPattonType();
    
    virtual UINT                    getBGGradationStartColor();
    virtual UINT                    getBGGradationEndColor();
    
    virtual BGGradationType            getBGGradationType();
    virtual UINT                    getBGGradationWidthCenter();
    virtual UINT                    getBGGradationHeightCenter();
    virtual UINT                    getBGGradationGradationangle();
    virtual UINT                    getBGGradationBlurlevel();
    virtual UINT                    getBGGradationBlurCenter();
    
    virtual std::string                getPictureFile();
    virtual bool                    getPictureInclude();
    virtual PicFillType                getPictureFillType();
    virtual JsonINT                    getPictureFillValue();
    virtual EffectType                getEffctType();
    virtual JsonINT                    getEffectValue();
    virtual UINT                    getWatermark();
    virtual bool                    getTableInTable();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// ISpecialCharacter    : 특수문자

__interface ISpecialCharacter : public ICheckListObj
{
    virtual UINT                    getMinimum();
    virtual UINT                    getMaximum();
};




////////////////////////////////////////////////////////////////////////////////////////////////
// IOutlineShape    : 개요 번호 모양

/*
    OUTLINE_NAME1            // 1. 가. 1) 가) (1) (가) ① ㉮
    OUTLINE_NAME2            // 공문서 - 들여 쓰기 없음(1. 가. 1) 가) (1) (가) ① ㉮)
    OUTLINE_NAME3            // I A 1 a (1) (a) ① ⓐ
    OUTLINE_NAME4            // 1. 가. ▶ (1) (가) - ① ㉮
    OUTLINE_NAME5            // I. A. 1. a) (1) (a) i)
    OUTLINE_NAME6            // 1. 1.1. 1.1.1. 1.1.1.1. ...
    OUTLINE_NAME7            // 1. 1.1 1.1.1 1.1.1.1 ...
    OUTLINE_NAME8            // 1. 가. 1. 가) (1) (가) i)
    OUTLINE_NAME9            // 제1장 제1절 제 1항 - ...
    OUTLINE_NAME10            // ● ■ ▶ ...
    OUTLINE_NAME11            // 1. 없음 없음 없음 ...
    OUTLINE_NAME12            // 없음 없음 없음 없음 ...

    DIGIT                    // 1, 2, 3
    CIRCLED_DIGIT            // 동그라미 쳐진 1, 2, 3
    ROMAN_CAPITAL            // I, II, III
    ROMAN_SMALL                // i, ii, iii
    LATIN_CAPITAL            // A, B, C
    LATIN_SMALL                // a, b, c
    CIRCLED_LATIN_CAPITAL    // 동그라미 쳐진 A, B, C
    CIRCLED_LATIN_SMALL        // 동그라미 쳐진 a, b, c
    HANGUL_SYLLABLE            // 가, 나, 다
    CIRCLED_HANGUL_SYLLABLE    // 동그라미 쳐진 가, 나, 다
    HANGUL_JAMO                // ㄱ, ㄴ, ㄷ
    CIRCLED_HANGUL_JAMO        // 동그라미 쳐진 ㄱ, ㄴ, ㄷ
    HANGUL_PHONETIC            // 일, 이, 삼
    IDEOGRAPH                // 一, 二, 三
    CIRCLED_IDEOGRAPH        // 동그라미 쳐진 一, 二, 三
    DECAGON_CIRCLE            // 갑, 을, 병, 정, 무, 기, 경, 신, 임, 계
    DECAGON_CIRCLE_HANJA    // 甲, 乙, 丙, 丁, 戊, 己, 庚, 辛, 壬, 癸
*/

enum OutlineType {
    OUTLINE_NAME1 = 0,
    OUTLINE_NAME2,
    OUTLINE_NAME3,
    OUTLINE_NAME4,
    OUTLINE_NAME5,
    OUTLINE_NAME6,
    OUTLINE_NAME7,
    OUTLINE_NAME8,
    OUTLINE_NAME9,
    OUTLINE_NAME10,
    OUTLINE_NAME11,
    OUTLINE_NAME12
};

enum NumberShapeType {
    DIGIT = 0,
    CIRCLED_DIGIT,
    ROMAN_CAPITAL,
    ROMAN_SMALL,
    LATIN_CAPITAL,
    LATIN_SMALL,
    CIRCLED_LATIN_CAPITAL,
    CIRCLED_LATIN_SMALL,
    HANGUL_SYLLABLE,
    CIRCLED_HANGUL_SYLLABLE,
    HANGUL_JAMO,
    CIRCLED_HANGUL_JAMO,
    HANGUL_PHONETIC,
    IDEOGRAPH,
    CIRCLED_IDEOGRAPH,
    DECAGON_CIRCLE,
    DECAGON_CIRCLE_HANJA
};

struct LevelType {
    UINT                        level;        // min 1, max 10
    std::string                    numberType;
    NumberShapeType                numShapeType;
    std::vector<UINT>            checkedList;    // 값의 설정 유무
};

__interface IOutlineShape : public ICheckListObj
{
    virtual OutlineType                getOutlineType();
    virtual bool                    getStartNumber();    // false : continue, true : newnumber
    virtual UINT                    getValue();            // check Min 1, max 10;
    virtual std::vector<LevelType*>    getLevelTypes();
};




////////////////////////////////////////////////////////////////////////////////////////////////
// IBullet    : 글머리표

enum BulletType {
    BULLETSHAPE_NONE = 0,
    BULLETSHAPE_NAME1,
    BULLETSHAPE_NAME2,
    BULLETSHAPE_NAME3,
    BULLETSHAPE_NAME4,
    BULLETSHAPE_NAME5,
    BULLETSHAPE_NAME6,
    BULLETSHAPE_NAME7,
    BULLETSHAPE_NAME8,
    BULLETSHAPE_NAME9,
    BULLETSHAPE_NAME10,
    BULLETSHAPE_NAME11,
    BULLETSHAPE_NAME12,
    BULLETSHAPE_NAME13,
    BULLETSHAPE_NAME14,
    BULLETSHAPE_NAME15,
    BULLETSHAPE_NAME16,
    BULLETSHAPE_NAME17,
    BULLETSHAPE_NAME18
};

enum CheckType {
    BULLETSHAPE_CHECK_NAME19 = 0,
    BULLETSHAPE_CHECK_NAME20,
    BULLETSHAPE_CHECK_NAME21,
    BULLETSHAPE_CHECK_NAME22,
    BULLETSHAPE_USER
};

__interface IBullet : public ICheckListObj
{
    virtual BulletType                getBulletType();
    virtual CheckType                getCheckType();
    virtual std::vector<UINT>        getCodes();
    virtual std::string                getBulletShapes();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// IParaNumBullet    : 문단 번호

/*
    PARANUMSHAPE_NAME1        1. 가. 1) 가) (1) (가) ① ㉮
    PARANUMSHAPE_NAME2        (1) (가) (a) 1) 가) a) ① ㉮ ⓐ
    PARANUMSHAPE_NAME3        1) 가) a) (1) (가) (a) ① ㉮ ⓐ
    PARANUMSHAPE_NAME4        ① (ㄱ) (a) 1) ㄱ) a)
    PARANUMSHAPE_NAME5        가) a) (1) (가) (a) ①
    PARANUMSHAPE_NAME6        (ㄱ) (1) (a) 1) a) ①
    PARANUMSHAPE_NAME7        I. A. 1. i) a) 1) ① ⓐ
    PARANUMSHAPE_NAME8        i a. (i) (a) i) a)
    PARANUMSHAPE_NAME9        A. 1. 가 (a) (1) (가) ⓐ ① ㉮
    PARANUMSHAPE_NAME10        1. 1.1. 1.1.1. 1.1.1.1. ...
*/

enum ParaNumType {
    PARANUMSHAPE_NAME1 = 0,
    PARANUMSHAPE_NAME2,
    PARANUMSHAPE_NAME3,
    PARANUMSHAPE_NAME4,
    PARANUMSHAPE_NAME5,
    PARANUMSHAPE_NAME6,
    PARANUMSHAPE_NAME7,
    PARANUMSHAPE_NAME8,
    PARANUMSHAPE_NAME9,
    PARANUMSHAPE_NAME10,
};

__interface IParaNumBullet : public ICheckListObj
{
    virtual ParaNumType                getParaNumType();
    virtual bool                    getStartNumber();    // false : continue, true : newnumber
    virtual UINT                    getValue();            // check Min 1, max 10;
    virtual std::vector<LevelType*>    getLevelTypes();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// IStyle    : 스타일

enum StyleType {
    StyleNormal = 0,
    StyleBody,
    StyleOutline1,
    StyleOutline2,
    StyleOutline3,
    StyleOutline4,
    StyleOutline5,
    StyleOutline6,
    StyleOutline7,
    StyleOutline8,
    StyleOutline9,
    StyleOutline10,
    StylePageNum,
    StyleHeader,
    StyleFootnote,
    StyleEndnote,
    StyleMemo,
    TOCStyle,
    TOC1Style,
    TOC2Style,
    TOC3Style
};

__interface IStyle : public ICheckListObj
{
    virtual StyleType                getStyleType();
    virtual bool                    getPermission();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// IHyperlink    : 하이퍼링크

__interface IHyperlink : public ICheckListObj
{
    virtual bool                    getPermission();
};


////////////////////////////////////////////////////////////////////////////////////////////////
// IMacro    : macro 사용

__interface IMacro : public ICheckListObj
{
    virtual bool                    getPermission();
};

