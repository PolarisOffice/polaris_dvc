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
#include <string>

#include "DVCInterface.h"
#include "JsonModel.h"

namespace CheckListModule {
    class CTable : public ITable
    {
    public:
        CTable(std::string dvcFile);
        CTable(LPCSTR document);
        ~CTable();

    public:
        DVCType getType() { return DVCType::Table; }
        std::map<std::string, int>    getMapData() { return m_mapCheckedList; }
        std::vector<UINT>        getCheckedList() { return m_checkedList; }

        JsonUINT                getSizeWidth();
        JsonUINT                getSizeHeight();
        bool                    getSizeFixed();
        bool                    getTreatAsChar();
        PosType                    getPosition();
        TextPosType                getTextPos();
        HorizontalType            getHType();
        HorizontalDirection        getHDirection();
        JsonINT                    getHValue();    // -1000 ~ 1000
        VerticalType            getVType();        
        VerticalDirection        getVDirection();
        JsonINT                    getVValue();    // -1000 ~ 1000
        bool                    getSoflowWithText();
        bool                    getSoalLowOverlap();
        bool                    getSoholdanchrObj();
        bool                    getParallel();
        JsonINT                    getRotation();
        JsonUINT                getGradientH();
        JsonUINT                getGradientV();
        NumType                    getNumverType();
        bool                    getObjProtect();
        JsonUINT                getMarginLeft();    // 0 ~ 100
        JsonUINT                getMarginRight();    // 0 ~ 100
        JsonUINT                getMarginTop();        // 0 ~ 100
        JsonUINT                getMarginBottom();    // 0 ~ 100
        CaptionPosType            getCaptionPosType();
        JsonUINT                getCaptionSize();        // 0 ~ 1000
        JsonUINT                getCaptionSpacing();    // -1000 ~ 1000
        bool                    getCaptionSocapFullSize();
        bool                    getCaptionLineWrap();

        BorderInfo*                getBorderInfo(LinePosition pos);
        std::vector<BorderInfo*> getBorderInfos();

        UINT                    getBorderCellSpacing();        // 0 ~ 49.99
        BGFillType                getBGFillType();
        UINT                    getBGFillFaceColor();
        UINT                    getBGFillPattonColor();
        BGPattonType            getBGFillPattonType();

        UINT                    getBGGradationStartColor();
        UINT                    getBGGradationEndColor();

        BGGradationType            getBGGradationType();
        UINT                    getBGGradationWidthCenter();    // 1~100
        UINT                    getBGGradationHeightCenter();    // 1~100
        UINT                    getBGGradationGradationangle();    // 0 ~ 359
        UINT                    getBGGradationBlurlevel();        // 0 ~ 255
        UINT                    getBGGradationBlurCenter();        // 0 ~ 100

        std::string                getPictureFile();
        bool                    getPictureInclude();
        PicFillType                getPictureFillType();
        JsonINT                    getPictureFillValue();            // -100 ~ 100
        EffectType                getEffctType();
        JsonINT                    getEffectValue();                // -100 ~ 100
        UINT                    getWatermark();
        bool                    getTableInTable();

    private:
        bool parsingDVCFile(std::string dvcFile);
        bool parsingDVCDocument(LPCSTR document);

        bool parsingElement(Json::Value root, void* Obj = 0);

        std::string                    m_dvcFile;
        std::vector<UINT>            m_checkedList;    // json 으로 설정되어 있는 항목들을 포함하고 있는 벡터. 이 값을 보고 실제 어떤 값들이 설정되어 있는지 확인한다.
        std::map<std::string, int>    m_mapCheckedList;

        JsonUINT                    m_sizeWidth;
        JsonUINT                    m_sizeHeight;
        bool                        m_sizeFixed;
        bool                        m_treatAsChar;
        PosType                        m_position;
        TextPosType                    m_textPos;
        HorizontalType                m_hType;
        HorizontalDirection            m_hDirection;
        JsonINT                        m_hValue;
        VerticalType                m_vType;
        VerticalDirection            m_vDirection;
        JsonINT                        m_vValue;
        bool                        m_soflowWithText;
        bool                        m_soalLowOverlap;
        bool                        m_soholdanchrObj;
        bool                        m_parallel;
        JsonINT                        m_rotation;
        JsonUINT                    m_gradientH;
        JsonUINT                    m_gradientV;
        NumType                        m_numverType;
        bool                        m_objProtect;
        JsonUINT                    m_marginLeft;
        JsonUINT                    m_marginRight;
        JsonUINT                    m_marginTop;
        JsonUINT                    m_marginBottom;
        CaptionPosType                m_captionPosType;
        JsonUINT                    m_captionSize;
        JsonUINT                    m_captionSpacing;
        bool                        m_captionSocapFullSize;
        bool                        m_captionLineWrap;

        std::vector<BorderInfo*>    m_borderInfo;

        UINT                        m_borderCellSpacing;
        BGFillType                    m_bgFillType;
        UINT                        m_bgFillFaceColor;
        UINT                        m_bgFillPattonColor;
        BGPattonType                m_bgFillPattonType;

        UINT                        m_bgGradationStartColor;
        UINT                        m_bgGradationEndColor;

        BGGradationType                m_bgGradationType;
        UINT                        m_bgGradationWidthCenter;
        UINT                        m_bgGradationHeightCenter;
        UINT                        m_bgGradationGradationangle;
        UINT                        m_bgGradationBlurlevel;
        UINT                        m_bgGradationBlurCenter;

        std::string                    m_pictureFile;
        bool                        m_pictureInclude;
        PicFillType                    m_pictureFillType;
        JsonINT                        m_pictureFillValue;
        EffectType                    m_effctType;
        JsonINT                        m_effectValue;
        UINT                        m_watermark;
        bool                        m_tableInTable;
    };
}


