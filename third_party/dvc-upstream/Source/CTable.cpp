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

#include "pch.h"
#include "CTable.h"

#include "JsonModel.h"

static std::map<std::string, int> tableMapData = {
    {JIN_TABLE,                                JID_TABLE},
    {JIN_TABLE_SIZEWIDTH,                    JID_TABLE_SIZEWIDTH},
    {JIN_TABLE_SIZEHEIGHT,                    JID_TABLE_SIZEHEIGHT},
    {JIN_TABLE_SIZEFIXED,                    JID_TABLE_SIZEFIXED},
    {JIN_TABLE_TREATASCHAR,                    JID_TABLE_TREATASCHAR},
    {JIN_TABLE_POS,                            JID_TABLE_POS},
    {JIN_TABLE_TEXTPOS,                        JID_TABLE_TEXTPOS},
    {JIN_TABLE_HTYPE,                        JID_TABLE_HTYPE},
    {JIN_TABLE_HDIRECTION,                    JID_TABLE_HDIRECTION},
    {JIN_TABLE_HVALUE,                        JID_TABLE_HVALUE},
    {JIN_TABLE_VTYPE,                        JID_TABLE_VTYPE},
    {JIN_TABLE_VDIRECTION,                    JID_TABLE_VDIRECTION},
    {JIN_TABLE_VVALUE,                        JID_TABLE_VVALUE},
    {JIN_TABLE_SOFLOWWITHTEXT,                JID_TABLE_SOFLOWWITHTEXT},
    {JIN_TABLE_SOALLOWOVERLAP,                JID_TABLE_SOALLOWOVERLAP},
    {JIN_TABLE_SOHOLDANCHOROBJ,                JID_TABLE_SOHOLDANCHOROBJ},
    {JIN_TABLE_PARALLEL,                    JID_TABLE_PARALLEL},
    {JIN_TABLE_ROTATION,                    JID_TABLE_ROTATION},
    {JIN_TABLE_GRADIENT_H,                    JID_TABLE_GRADIENT_H},
    {JIN_TABLE_GRADIENT_V,                    JID_TABLE_GRADIENT_V},
    {JIN_TABLE_NUMVERTYPE,                    JID_TABLE_NUMVERTYPE},
    {JIN_TABLE_OBJPROTECT,                    JID_TABLE_OBJPROTECT},
    {JIN_TABLE_MARGIN_LEFT,                    JID_TABLE_MARGIN_LEFT},
    {JIN_TABLE_MARGIN_RIGHT,                JID_TABLE_MARGIN_RIGHT},
    {JIN_TABLE_MARGIN_TOP,                    JID_TABLE_MARGIN_TOP},
    {JIN_TABLE_MARGIN_BOTTOM,                JID_TABLE_MARGIN_BOTTOM},
    {JIN_TABLE_CAPTION_POSITION,            JID_TABLE_CAPTION_POSITION},
    {JIN_TABLE_CAPTION_SIZE,                JID_TABLE_CAPTION_SIZE},
    {JIN_TABLE_CAPTION_SPACING,                JID_TABLE_CAPTION_SPACING},
    {JIN_TABLE_CAPTION_SOCAPFULLSIZE,        JID_TABLE_CAPTION_SOCAPFULLSIZE},
    {JIN_TABLE_CAPTION_LINEWRAP,            JID_TABLE_CAPTION_LINEWRAP},
    {JIN_TABLE_BORDER,                        JID_TABLE_BORDER},
    {JIN_TABLE_BORDER_POSITION,                JID_TABLE_BORDER_POSITION},
    {JIN_TABLE_BORDER_TYPE,                    JID_TABLE_BORDER_TYPE},
    {JIN_TABLE_BORDER_SIZE,                    JID_TABLE_BORDER_SIZE},
    {JIN_TABLE_BORDER_COLOR,                JID_TABLE_BORDER_COLOR},
    {JIN_TABLE_BORDER_CELLSPACING,            JID_TABLE_BORDER_CELLSPACING},
    {JIN_TABLE_BGFILL_TYPE,                    JID_TABLE_BGFILL_TYPE},
    {JIN_TABLE_BGFILL_FACECOLOR,            JID_TABLE_BGFILL_FACECOLOR},
    {JIN_TABLE_BGFILL_PATTONCOLOR,            JID_TABLE_BGFILL_PATTONCOLOR},
    {JIN_TABLE_BGFILL_PATTONTYPE,            JID_TABLE_BGFILL_PATTONTYPE},
    {JIN_TABLE_BGGRADATION_STARTCOLOR,        JID_TABLE_BGGRADATION_STARTCOLOR},
    {JIN_TABLE_BGGRADATION_ENDCOLOR,        JID_TABLE_BGGRADATION_ENDCOLOR},
    {JIN_TABLE_BGGRADATION_TYPE,            JID_TABLE_BGGRADATION_TYPE},
    {JIN_TABLE_BGGRADATION_WIDTHCENTER,        JID_TABLE_BGGRADATION_WIDTHCENTER},
    {JIN_TABLE_BGGRADATION_HEIGHTCENTER,    JID_TABLE_BGGRADATION_HEIGHTCENTER},
    {JIN_TABLE_BGGRADATION_GRADATIONANGLE,    JID_TABLE_BGGRADATION_GRADATIONANGLE},
    {JIN_TABLE_BGGRADATION_BLURLEVEL,        JID_TABLE_BGGRADATION_BLURLEVEL},
    {JIN_TABLE_BGGRADATION_BLURCENTER,        JID_TABLE_BGGRADATION_BLURCENTER},
    {JIN_TABLE_PICTURE_FILE,                JID_TABLE_PICTURE_FILE},
    {JIN_TABLE_PICTURE_INCLUDE,                JID_TABLE_PICTURE_INCLUDE},
    {JIN_TABLE_PICTUREFILL_TYPE,            JID_TABLE_PICTUREFILL_TYPE},
    {JIN_TABLE_PICTUREFILL_VALUE,            JID_TABLE_PICTUREFILL_VALUE},
    {JIN_TABLE_EFFECT_TYPE,                    JID_TABLE_EFFECT_TYPE},
    {JIN_TABLE_EFFECT_VALUE,                JID_TABLE_EFFECT_VALUE},
    {JIN_TABLE_WATERMARK,                    JID_TABLE_WATERMARK},
    {JIN_TABLE_TABLE_IN_TABLE,                JID_TABLE_TABLE_IN_TABLE}    
};

namespace CheckListModule {

    CTable::CTable(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    CTable::CTable(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CTable::~CTable()
    {
        if (m_borderInfo.size() > 0)
        {
            std::vector<BorderInfo*>::iterator iter = m_borderInfo.begin();

            for (iter; iter != m_borderInfo.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_borderInfo.clear();
        }
    }

    JsonUINT CTable::getSizeWidth()
    {
        return m_sizeWidth;
    }

    JsonUINT CTable::getSizeHeight()
    {
        return m_sizeHeight;
    }

    bool CTable::getSizeFixed()
    {
        return m_sizeFixed;
    }

    bool CTable::getTreatAsChar()
    {
        return m_treatAsChar;
    }

    PosType CTable::getPosition()
    {
        return m_position;
    }

    TextPosType CTable::getTextPos()
    {
        return m_textPos;
    }

    HorizontalType CTable::getHType()
    {
        return m_hType;
    }

    HorizontalDirection CTable::getHDirection()
    {
        return m_hDirection;
    }

    JsonINT CTable::getHValue()
    {
        return m_hValue;
    }

    VerticalType CTable::getVType()
    {
        return m_vType;
    }

    VerticalDirection CTable::getVDirection()
    {
        return m_vDirection;
    }

    JsonINT CTable::getVValue()
    {
        return m_vValue;
    }

    bool CTable::getSoflowWithText()
    {
        return m_soflowWithText;
    }

    bool CTable::getSoalLowOverlap()
    {
        return m_soalLowOverlap;
    }

    bool CTable::getSoholdanchrObj()
    {
        return m_soholdanchrObj;
    }

    bool CTable::getParallel()
    {
        return m_parallel;
    }

    JsonINT CTable::getRotation()
    {
        return m_rotation;
    }

    JsonUINT CTable::getGradientH()
    {
        return m_gradientH;
    }

    JsonUINT CTable::getGradientV()
    {
        return m_gradientV;
    }

    NumType CTable::getNumverType()
    {
        return m_numverType;
    }

    bool CTable::getObjProtect()
    {
        return m_objProtect;
    }

    JsonUINT CTable::getMarginLeft()
    {
        return m_marginLeft;
    }

    JsonUINT CTable::getMarginRight()
    {
        return m_marginRight;
    }

    JsonUINT CTable::getMarginTop()
    {
        return m_marginTop;
    }

    JsonUINT CTable::getMarginBottom()
    {
        return m_marginBottom;
    }

    CaptionPosType CTable::getCaptionPosType()
    {
        return m_captionPosType;
    }

    JsonUINT CTable::getCaptionSize()
    {
        return m_captionSize;
    }

    JsonUINT CTable::getCaptionSpacing()
    {
        return m_captionSpacing;
    }

    bool CTable::getCaptionSocapFullSize()
    {
        return m_captionSocapFullSize;
    }

    bool CTable::getCaptionLineWrap()
    {
        return m_captionLineWrap;
    }


    BorderInfo* CTable::getBorderInfo(LinePosition pos)
    {
        int i = 0;
        for (i = 0; i < m_borderInfo.size(); i++)
        {
            if (m_borderInfo.at(i)->position == pos)
                return m_borderInfo.at(i);
        }
        return NULL;
    }

    std::vector<BorderInfo*> CTable::getBorderInfos()
    {
        return m_borderInfo;
    }

    UINT CTable::getBorderCellSpacing()
    {
        return m_borderCellSpacing;
    }

    BGFillType CTable::getBGFillType()
    {
        return m_bgFillType;
    }

    UINT CTable::getBGFillFaceColor()
    {
        return m_bgFillFaceColor;
    }

    UINT CTable::getBGFillPattonColor()
    {
        return m_bgFillPattonColor;
    }

    BGPattonType CTable::getBGFillPattonType()
    {
        return m_bgFillPattonType;
    }


    UINT CTable::getBGGradationStartColor()
    {
        return m_bgGradationStartColor;
    }

    UINT CTable::getBGGradationEndColor()
    {
        return m_bgGradationEndColor;
    }


    BGGradationType CTable::getBGGradationType()
    {
        return m_bgGradationType;
    }

    UINT CTable::getBGGradationWidthCenter()
    {
        return m_bgGradationWidthCenter;
    }

    UINT CTable::getBGGradationHeightCenter()
    {
        return m_bgGradationHeightCenter;
    }

    UINT CTable::getBGGradationGradationangle()
    {
        return m_bgGradationGradationangle;
    }

    UINT CTable::getBGGradationBlurlevel()
    {
        return m_bgGradationBlurlevel;
    }

    UINT CTable::getBGGradationBlurCenter()
    {
        return m_bgGradationBlurCenter;
    }


    std::string CTable::getPictureFile()
    {
        return m_pictureFile;
    }

    bool CTable::getPictureInclude()
    {
        return m_pictureInclude;
    }

    PicFillType CTable::getPictureFillType()
    {
        return m_pictureFillType;
    }

    JsonINT CTable::getPictureFillValue()
    {
        return m_pictureFillValue;
    }

    EffectType CTable::getEffctType()
    {
        return m_effctType;
    }

    JsonINT CTable::getEffectValue()
    {
        return m_effectValue;
    }

    UINT CTable::getWatermark()
    {
        return m_watermark;
    }

    bool CTable::getTableInTable()
    {
        return m_tableInTable;
    }

    bool CTable::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_TABLE];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CTable::parsingDVCDocument(LPCSTR document)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;

        if (reader.parse(document, root))
        {
            bResult = parsingElement(root);
        }

        return bResult;
    }

    bool CTable::parsingElement(Json::Value root, void* Obj)
    {
        std::map<std::string, int>::iterator mapIter;
        Json::ValueIterator it = root.begin();

        bool bFind = false;

        while (it != root.end())
        {
            int nValue = 0;
            bool bValue = false;
            std::string strValue = "";
            Json::Value jsonValue;

            bFind = true;

            if (root[it.name().c_str()].isInt())
                nValue = root[it.name().c_str()].asInt();
            else if (root[it.name().c_str()].isBool())
                bValue = root[it.name().c_str()].asBool();
            else if (root[it.name().c_str()].isString())
                strValue = root[it.name().c_str()].asString();
            else if (root[it.name().c_str()].isObject())
                jsonValue = root[it.name().c_str()];            

            mapIter = tableMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_TABLE_SIZEWIDTH:
                m_sizeWidth.min = jsonValue[JNI_INT_MIN].asInt();
                m_sizeWidth.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_sizeWidth.min == m_sizeWidth.max)
                    m_sizeWidth.value = m_sizeWidth.min;
                break;
            case JID_TABLE_SIZEHEIGHT:
                m_sizeHeight.min = jsonValue[JNI_INT_MIN].asInt();
                m_sizeHeight.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_sizeHeight.min == m_sizeHeight.max)
                    m_sizeHeight.value = m_sizeHeight.min;
                break;
            case JID_TABLE_SIZEFIXED:
                m_sizeFixed = bValue;
                break;
            case JID_TABLE_TREATASCHAR:
                m_treatAsChar = bValue;
                break;
            case JID_TABLE_POS:
                m_position = (PosType)nValue;
                break;
            case JID_TABLE_TEXTPOS:
                m_textPos = (TextPosType)nValue;
                break;
            case JID_TABLE_HTYPE:
                m_hType = (HorizontalType)nValue;
                break;
            case JID_TABLE_HDIRECTION:
                m_hDirection = (HorizontalDirection)nValue;
                break;
            case JID_TABLE_HVALUE:
                m_hValue.min = jsonValue[JNI_INT_MIN].asInt();
                m_hValue.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_hValue.min == m_hValue.max)
                    m_hValue.value = m_hValue.min;
                break;
            case JID_TABLE_VTYPE:
                m_vType = (VerticalType)nValue;
                break;
            case JID_TABLE_VDIRECTION:
                m_vDirection = (VerticalDirection)nValue;
                break;
            case JID_TABLE_VVALUE:
                m_vValue.min = jsonValue[JNI_INT_MIN].asInt();
                m_vValue.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_vValue.min == m_vValue.max)
                    m_vValue.value = m_vValue.min;
                break;
            case JID_TABLE_SOFLOWWITHTEXT:
                m_soflowWithText = bValue;
                break;
            case JID_TABLE_SOALLOWOVERLAP:
                m_soalLowOverlap = bValue;
                break;
            case JID_TABLE_SOHOLDANCHOROBJ:
                m_soholdanchrObj = bValue;
                break;
            case JID_TABLE_PARALLEL:
                m_parallel = bValue;
                break;
            case JID_TABLE_ROTATION:
                m_rotation.min = jsonValue[JNI_INT_MIN].asInt();
                m_rotation.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_rotation.min == m_rotation.max)
                    m_rotation.value = m_rotation.min;
                break;
            case JID_TABLE_GRADIENT_H:
                m_gradientH.min = jsonValue[JNI_INT_MIN].asInt();
                m_gradientH.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_gradientH.min == m_gradientH.max)
                    m_gradientH.value = m_gradientH.min;
                break;
            case JID_TABLE_GRADIENT_V:
                m_gradientV.min = jsonValue[JNI_INT_MIN].asInt();
                m_gradientV.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_gradientV.min == m_gradientV.max)
                    m_gradientV.value = m_gradientV.min;
                break;
            case JID_TABLE_NUMVERTYPE:
                m_numverType = (NumType)nValue;
                break;
            case JID_TABLE_OBJPROTECT:
                m_objProtect = bValue;
                break;
            case JID_TABLE_MARGIN_LEFT:
                m_marginLeft.min = jsonValue[JNI_INT_MIN].asInt();
                m_marginLeft.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_marginLeft.min == m_marginLeft.max)
                    m_marginLeft.value = m_marginLeft.min;
                break;
            case JID_TABLE_MARGIN_RIGHT:
                m_marginRight.min = jsonValue[JNI_INT_MIN].asInt();
                m_marginRight.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_marginRight.min == m_marginRight.max)
                    m_marginRight.value = m_marginRight.min;
                break;
            case JID_TABLE_MARGIN_TOP:
                m_marginTop.min = jsonValue[JNI_INT_MIN].asInt();
                m_marginTop.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_marginTop.min == m_marginTop.max)
                    m_marginTop.value = m_marginTop.min;
                break;
            case JID_TABLE_MARGIN_BOTTOM:
                m_marginBottom.min = jsonValue[JNI_INT_MIN].asInt();
                m_marginBottom.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_marginBottom.min == m_marginBottom.max)
                    m_marginBottom.value = m_marginBottom.min;
                break;
            case JID_TABLE_CAPTION_POSITION:
                m_captionPosType = (CaptionPosType)nValue;
                break;
            case JID_TABLE_CAPTION_SIZE:
                m_captionSize.min = jsonValue[JNI_INT_MIN].asInt();
                m_captionSize.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_captionSize.min == m_captionSize.max)
                    m_captionSize.value = m_captionSize.min;
                break;
            case JID_TABLE_CAPTION_SPACING:
                m_captionSpacing.min = jsonValue[JNI_INT_MIN].asInt();
                m_captionSpacing.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_captionSpacing.min == m_captionSpacing.max)
                    m_captionSpacing.value = m_captionSpacing.min;
                break;
            case JID_TABLE_CAPTION_SOCAPFULLSIZE:
                m_captionSocapFullSize = bValue;
                break;
            case JID_TABLE_CAPTION_LINEWRAP:
                m_captionLineWrap = bValue;
                break;
            case JID_TABLE_BORDER:
            {
                BorderInfo* borderInfo = NULL;

                auto value = root[it.name().c_str()];

                for (int i = 0; i < (int)value.size(); i++)
                {
                    borderInfo = new BorderInfo();

                    if (borderInfo)
                    {
                        Json::Value childValue = value[i];

                        parsingElement(childValue, borderInfo);
                        m_borderInfo.push_back(borderInfo);
                    }
                }
                break;
            }
            case JID_TABLE_BORDER_POSITION:
                if (Obj != NULL)
                {
                    ((BorderInfo*)Obj)->position = (LinePosition)nValue;
                    ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_TABLE_BORDER_TYPE:
                if (Obj != NULL)
                {
                    ((BorderInfo*)Obj)->lineShape = (LineShape)nValue;
                    ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_TABLE_BORDER_SIZE:
                if (Obj != NULL)
                {
                    ((BorderInfo*)Obj)->size = root[it.name().c_str()].asFloat();
                    ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_TABLE_BORDER_COLOR:
                if (Obj != NULL)
                {
                    ((BorderInfo*)Obj)->color = nValue;
                    ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_TABLE_BORDER_CELLSPACING:
                m_borderCellSpacing = nValue;
                break;
            case JID_TABLE_BGFILL_TYPE:
                m_bgFillType = (BGFillType)nValue;
                break;
            case JID_TABLE_BGFILL_FACECOLOR:
                m_bgFillFaceColor = nValue;
                break;
            case JID_TABLE_BGFILL_PATTONCOLOR:
                m_bgFillPattonColor = nValue;
                break;
            case JID_TABLE_BGFILL_PATTONTYPE:
                m_bgFillPattonType = (BGPattonType)nValue;
                break;
            case JID_TABLE_BGGRADATION_STARTCOLOR:
                m_bgGradationStartColor = nValue;
                break;
            case JID_TABLE_BGGRADATION_ENDCOLOR:
                m_bgGradationEndColor = nValue;
                break;
            case JID_TABLE_BGGRADATION_TYPE:
                m_bgGradationType = (BGGradationType)nValue;
                break;
            case JID_TABLE_BGGRADATION_WIDTHCENTER:
                m_bgGradationWidthCenter = nValue;
                break;
            case JID_TABLE_BGGRADATION_HEIGHTCENTER:
                m_bgGradationHeightCenter = nValue;
                break;
            case JID_TABLE_BGGRADATION_GRADATIONANGLE:
                m_bgGradationGradationangle = nValue;
                break;
            case JID_TABLE_BGGRADATION_BLURLEVEL:
                m_bgGradationBlurlevel = nValue;
                break;
            case JID_TABLE_BGGRADATION_BLURCENTER:
                m_bgGradationBlurCenter = nValue;
                break;
            case JID_TABLE_PICTURE_FILE:
                m_pictureFile = strValue;
                break;
            case JID_TABLE_PICTURE_INCLUDE:
                m_pictureInclude = bValue;
                break;
            case JID_TABLE_PICTUREFILL_TYPE:
                m_pictureFillType = (PicFillType)nValue;
                break;
            case JID_TABLE_PICTUREFILL_VALUE:
                m_pictureFillValue.min = jsonValue[JNI_INT_MIN].asInt();
                m_pictureFillValue.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_pictureFillValue.min == m_pictureFillValue.max)
                    m_pictureFillValue.value = m_pictureFillValue.min;
                break;
            case JID_TABLE_EFFECT_TYPE:
                m_effctType = (EffectType)nValue;
                break;
            case JID_TABLE_EFFECT_VALUE:
                m_effectValue.min = jsonValue[JNI_INT_MIN].asInt();
                m_effectValue.max = jsonValue[JNI_INT_MAX].asInt();
                if (m_effectValue.min == m_effectValue.max)
                    m_effectValue.value = m_effectValue.min;
                break;
            case JID_TABLE_WATERMARK:
                m_watermark = nValue;
                break;
            case JID_TABLE_TABLE_IN_TABLE:
                m_tableInTable = bValue;
                break;
            default:
                break;
            }

            if (bFind)
            {
                m_checkedList.push_back(mapIter->second);
                m_mapCheckedList.insert(std::map<std::string, int>::value_type(mapIter->first, mapIter->second));
            }

            it++;
        }

        return true;
    }
}