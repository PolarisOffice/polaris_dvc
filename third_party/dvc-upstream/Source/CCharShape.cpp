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
#include "CCharShape.h"

#include "JsonModel.h"

#include <iostream>
#include <codecvt>

static std::map<std::string, int> charshapeMapData = {
    {JIN_CHAR_SHAPE, JID_CHAR_SHAPE},
    {JIN_CHAR_SHAPE_FONTSIZE, JID_CHAR_SHAPE_FONTSIZE},
    {JIN_CHAR_SHAPE_LANGSET, JID_CHAR_SHAPE_LANGSET},
    {JIN_CHAR_SHAPE_LANG, JID_CHAR_SHAPE_LANG},
    {JIN_CHAR_SHAPE_FONT, JID_CHAR_SHAPE_FONT},
    {JIN_CHAR_SHAPE_RSIZE, JID_CHAR_SHAPE_RSIZE},
    {JIN_CHAR_SHAPE_POSITION, JID_CHAR_SHAPE_POSITION},
    {JIN_CHAR_SHAPE_RATIO, JID_CHAR_SHAPE_RATIO},
    {JIN_CHAR_SHAPE_SPACING, JID_CHAR_SHAPE_SPACING},
    {JIN_CHAR_SHAPE_BOLD, JID_CHAR_SHAPE_BOLD},
    {JIN_CHAR_SHAPE_ITALIC, JID_CHAR_SHAPE_ITALIC},
    {JIN_CHAR_SHAPE_UNDERLINE, JID_CHAR_SHAPE_UNDERLINE},
    {JIN_CHAR_SHAPE_STRIKEOUT, JID_CHAR_SHAPE_STRIKEOUT},
    {JIN_CHAR_SHAPE_OUTLINE, JID_CHAR_SHAPE_OUTLINE},
    {JIN_CHAR_SHAPE_EMBOSS, JID_CHAR_SHAPE_EMBOSS},
    {JIN_CHAR_SHAPE_ENGRAVE, JID_CHAR_SHAPE_ENGRAVE},
    {JIN_CHAR_SHAPE_SHADOW, JID_CHAR_SHAPE_SHADOW},
    {JIN_CHAR_SHAPE_SUPSCRIPT, JID_CHAR_SHAPE_SUPSCRIPT},
    {JIN_CHAR_SHAPE_SUBSCRIPT, JID_CHAR_SHAPE_SUBSCRIPT},
    {JIN_CHAR_SHAPE_SHADOWTYPE, JID_CHAR_SHAPE_SHADOWTYPE},
    {JIN_CHAR_SHAPE_SHADOW_X, JID_CHAR_SHAPE_SHADOW_X},
    {JIN_CHAR_SHAPE_SHADOW_Y, JID_CHAR_SHAPE_SHADOW_Y},
    {JIN_CHAR_SHAPE_SHADOW_COLOR, JID_CHAR_SHAPE_SHADOW_COLOR},
    {JIN_CHAR_SHAPE_UNDERLINE_POSITOION, JID_CHAR_SHAPE_UNDERLINE_POSITOION},
    {JIN_CHAR_SHAPE_UNDERLINE_SHAPE, JID_CHAR_SHAPE_UNDERLINE_SHAPE},
    {JIN_CHAR_SHAPE_UNDERLINE_COLOR, JID_CHAR_SHAPE_UNDERLINE_COLOR},
    {JIN_CHAR_SHAPE_STRIKEOUT_SHAPE, JID_CHAR_SHAPE_STRIKEOUT_SHAPE},
    {JIN_CHAR_SHAPE_STRIKEOUT_COLOR, JID_CHAR_SHAPE_STRIKEOUT_COLOR},
    {JIN_CHAR_SHAPE_OUTLINETYPE, JID_CHAR_SHAPE_OUTLINETYPE},
    {JIN_CHAR_SHAPE_EMPTYSPACE, JID_CHAR_SHAPE_EMPTYSPACE},
    {JIN_CHAR_SHAPE_POINT, JID_CHAR_SHAPE_POINT},
    {JIN_CHAR_SHAPE_KERNING, JID_CHAR_SHAPE_KERNING},
    {JIN_CHAR_SHAPE_BG_BORDER, JID_CHAR_SHAPE_BG_BORDER},
    {JIN_CHAR_SHAPE_BG_BORDER_POSITION, JID_CHAR_SHAPE_BG_BORDER_POSITION},
    {JIN_CHAR_SHAPE_BG_BORDER_BORDERTYPE,JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE},
    {JIN_CHAR_SHAPE_BG_BORDER_SIZE, JID_CHAR_SHAPE_BG_BORDER_SIZE},
    {JIN_CHAR_SHAPE_BG_BORDER_COLOR, JID_CHAR_SHAPE_BG_BORDER_COLOR},
    {JIN_CHAR_SHAPE_BG_COLOR, JID_CHAR_SHAPE_BG_COLOR},
    {JIN_CHAR_SHAPE_BG_PATTONCOLOR, JID_CHAR_SHAPE_BG_PATTONCOLOR},
    {JIN_CHAR_SHAPE_BG_PATTONTYPE, JID_CHAR_SHAPE_BG_PATTONTYPE}    
};


namespace CheckListModule {

    CCharShape::CCharShape(std::string dvcFile)
    {
        m_dvcFile = dvcFile;

        parsingDVCFile(m_dvcFile);        
    }

    CCharShape::CCharShape(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CCharShape::~CCharShape()
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

    JsonUINT CCharShape::getFontSize()
    {
        if (m_fontSize.min == 0 && m_fontSize.max == 0)
        {
            m_fontSize.min = m_fontSize.value;
            m_fontSize.max = m_fontSize.value;
        }
        return m_fontSize;
    }

    LangType CCharShape::getLangType()
    {
        return m_langType;
    }

    std::vector<std::string> CCharShape::getFontNames()
    {
        return m_fontName;
    }

    JsonUINT CCharShape::getRSize()
    {
        if (m_rSize.min == 0 && m_rSize.max == 0)
        {
            m_rSize.min = m_rSize.value;
            m_rSize.max = m_rSize.value;
        }
        return m_rSize;
    }

    JsonINT CCharShape::getPosition()
    {
        if (m_position.min == 0 && m_position.max == 0)
        {
            m_position.min = m_position.value;
            m_position.max = m_position.value;
        }
        return m_position;
    }

    JsonUINT CCharShape::getRatio()
    {
        if (m_ratio.min == 0 && m_ratio.max == 0)
        {
            m_ratio.min = m_ratio.value;
            m_ratio.max = m_ratio.value;
        }
        return m_ratio;
    }

    JsonINT CCharShape::getSpacing()
    {
        if (m_spacing.min == 0 && m_spacing.max == 0)
        {
            m_spacing.min = m_spacing.value;
            m_spacing.max = m_spacing.value;
        }
        return m_spacing;
    }
        
    bool CCharShape::getBold()
    {
        return m_bBold;
    }

    bool CCharShape::getItalic()
    {
        return m_bItalic;
    }

    bool CCharShape::getUnderline()
    {
        return m_bUnderline;
    }

    bool CCharShape::getStrikeout()
    {
        return m_bStrikeout;
    }

    bool CCharShape::getOoutline()
    {
        return m_bOoutline;
    }

    bool CCharShape::getEmboss()
    {
        return m_bEmboss;
    }

    bool CCharShape::getEngrave()
    {
        return m_bEngrave;
    }

    bool CCharShape::getShadow()
    {
        return m_bShadow;
    }

    bool CCharShape::getSupscript()
    {
        return m_bSupscript;
    }

    bool CCharShape::getSubscript()
    {
        return m_bSubscript;
    }

    ShadowType CCharShape::getShadowType()
    {
        return m_shadowType;
    }

    INT CCharShape::getShadowXDirection()
    {
        return m_shadowXDirection.value;
    }

    INT CCharShape::getShadowYDirection()
    {
        return m_shadowYDirection.value;
    }

    UINT CCharShape::getShadowColor()
    {
        return m_shadowColor;
    }


    LinePosition CCharShape::getUnderlinePos()
    {
        return m_underlinePos;
    }

    LineShape CCharShape::getUnderlineShape()
    {
        return m_underlineShape;
    }

    UINT CCharShape::getUnderlineColor()
    {
        return m_underlineColor;
    }


    LineShape CCharShape::getStrikeoutShape()
    {
        return m_strikeoutShape;
    }

    UINT CCharShape::getStrikeColor()
    {
        return m_strikeColor;
    }


    LineShape CCharShape::getOutlineShape()
    {
        return m_outlineShape;
    }

    bool CCharShape::getEmptyspace()
    {
        return m_emptyspace;
    }

    UINT CCharShape::getPoint()
    {
        return m_point;
    }

    bool CCharShape::getKerning()
    {
        return m_kerning;
    }


    BorderInfo* CCharShape::getBorderInfo(LinePosition pos)
    {
        int i = 0;
        for (i = 0; i < m_borderInfo.size(); i++)
        {
            if (m_borderInfo.at(i)->position == pos)
                return m_borderInfo.at(i);
        }
        return NULL;
    }

    std::vector<BorderInfo*> CCharShape::getBorderInfos()
    {
        return m_borderInfo;
    }

    UINT CCharShape::getBackgroundColor()
    {
        return m_backgroundColor;
    }

    UINT CCharShape::getPattonColor()
    {
        return m_pattonColor;
    }

    PattonType CCharShape::getPattonType()
    {
        return m_pattonType;
    }

    bool CCharShape::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;
        
        if (reader.parse(document, root))
        {            
            Json::Value charshape = root[JIN_CHAR_SHAPE];
            
            bResult = parsingElement(charshape);
        }

        return bResult;
    }

    bool CCharShape::parsingDVCDocument(LPCSTR document)
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

    bool CCharShape::parsingElement(Json::Value root, void* Obj)    // root = root[JIN_CHAR_SHAPE]
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
            else
                jsonValue = root[it.name().c_str()];

            mapIter = charshapeMapData.find(it.key().asString());    // 또는 it.key()로 해보자.
            //std::cout << it.key() << std::endl;
            if (mapIter != charshapeMapData.end())
            {
                switch (mapIter->second)
                {
                case JID_CHAR_SHAPE:
                    break;
                case JID_CHAR_SHAPE_FONTSIZE:                    
                    m_fontSize.min = jsonValue[JNI_INT_MIN].asInt();
                    m_fontSize.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_fontSize.min == m_fontSize.max)
                        m_fontSize.value = m_fontSize.min;
                    break;
                case JID_CHAR_SHAPE_LANG:
                    m_langType = (LangType)nValue;                            
                    break;
                case JID_CHAR_SHAPE_FONT:
                    for (int j = 0; j < jsonValue.size(); j++)
                    {                                                
                        if (jsonValue[j].isString())
                        {
                            const std::string name = jsonValue[j].asString();
                            m_fontName.push_back(name);
                        }                        
                    }
                    break;
                case JID_CHAR_SHAPE_RSIZE:
                    m_rSize.min = jsonValue[JNI_INT_MIN].asInt();
                    m_rSize.max = jsonValue[JNI_INT_MAX].asInt();                    
                    if (m_rSize.min == m_rSize.max)
                        m_rSize.value = m_rSize.min;
                    break;
                case JID_CHAR_SHAPE_POSITION:
                    if (root[it.name().c_str()].isInt())
                        m_position.value = nValue;                        
                    else {
                        m_position.min = jsonValue[JNI_INT_MIN].asInt();
                        m_position.max = jsonValue[JNI_INT_MAX].asInt();
                        if (m_position.min == m_position.max)
                            m_position.value = m_position.min;
                    }
                    break;
                case JID_CHAR_SHAPE_RATIO:
                    if (root[it.name().c_str()].isInt())
                        m_ratio.value = nValue;
                    else {
                        m_ratio.min = jsonValue[JNI_INT_MIN].asInt();
                        m_ratio.max = jsonValue[JNI_INT_MAX].asInt();
                        if (m_ratio.min == m_ratio.max)
                            m_ratio.value = m_ratio.min;
                    }
                    break;
                case JID_CHAR_SHAPE_SPACING:
                    if (root[it.name().c_str()].isInt())
                        m_spacing.value = nValue;
                    else {
                        m_spacing.min = jsonValue[JNI_INT_MIN].asInt();
                        m_spacing.max = jsonValue[JNI_INT_MAX].asInt();
                        if (m_spacing.min == m_spacing.max)
                            m_spacing.value = m_spacing.min;
                    }
                    break;
                case JID_CHAR_SHAPE_BOLD:
                    m_bBold = bValue;
                    break;
                case JID_CHAR_SHAPE_ITALIC:
                    m_bItalic = bValue;
                    break;
                case JID_CHAR_SHAPE_UNDERLINE:
                    m_bUnderline = bValue;
                    break;
                case JID_CHAR_SHAPE_STRIKEOUT:
                    m_bStrikeout = bValue;
                    break;
                case JID_CHAR_SHAPE_OUTLINE:
                    m_bOoutline = bValue;
                    break;
                case JID_CHAR_SHAPE_EMBOSS:
                    m_bEmboss = bValue;
                    break;
                case JID_CHAR_SHAPE_ENGRAVE:
                    m_bEngrave = bValue;
                    break;
                case JID_CHAR_SHAPE_SHADOW:
                    m_bShadow = bValue;
                    break;
                case JID_CHAR_SHAPE_SUPSCRIPT:
                    m_bSupscript = bValue;
                    break;
                case JID_CHAR_SHAPE_SUBSCRIPT:
                    m_bSubscript = bValue;
                    break;
                case JID_CHAR_SHAPE_SHADOWTYPE:
                    m_shadowType = (ShadowType)nValue;
                    break;
                case JID_CHAR_SHAPE_SHADOW_X:
                    if (root[it.name().c_str()].isInt())
                        m_shadowXDirection.value = nValue;
                    else {
                        m_shadowXDirection.min = jsonValue[JNI_INT_MIN].asInt();
                        m_shadowXDirection.max = jsonValue[JNI_INT_MAX].asInt();
                        if (m_shadowXDirection.min == m_shadowXDirection.max)
                            m_shadowXDirection.value = m_shadowXDirection.min;
                    }
                    break;
                case JID_CHAR_SHAPE_SHADOW_Y:
                    if (root[it.name().c_str()].isInt())
                        m_shadowYDirection.value = nValue;
                    else {
                        m_shadowYDirection.min = jsonValue[JNI_INT_MIN].asInt();
                        m_shadowYDirection.max = jsonValue[JNI_INT_MAX].asInt();
                        if (m_shadowYDirection.min == m_shadowYDirection.max)
                            m_shadowYDirection.value = m_shadowYDirection.min;
                    }
                    break;
                case JID_CHAR_SHAPE_SHADOW_COLOR:
                    m_shadowColor = nValue;
                    break;
                case JID_CHAR_SHAPE_UNDERLINE_POSITOION:
                    m_underlinePos = (LinePosition)nValue;
                    break;
                case JID_CHAR_SHAPE_UNDERLINE_SHAPE:
                    m_underlineShape = (LineShape)nValue;
                    break;
                case JID_CHAR_SHAPE_UNDERLINE_COLOR:
                    m_underlineColor = nValue;
                    break;
                case JID_CHAR_SHAPE_STRIKEOUT_SHAPE:
                    m_strikeoutShape = (LineShape)nValue;
                    break;
                case JID_CHAR_SHAPE_STRIKEOUT_COLOR:
                    m_strikeColor = nValue;
                    break;
                case JID_CHAR_SHAPE_OUTLINETYPE:
                    m_outlineShape = (LineShape)nValue;
                    break;
                case JID_CHAR_SHAPE_EMPTYSPACE:
                    m_emptyspace = bValue;
                    break;
                case JID_CHAR_SHAPE_POINT:
                    m_point = nValue;    // m_point 타입을 확인해보자
                    break;
                case JID_CHAR_SHAPE_KERNING:
                    m_kerning = bValue;
                    break;
                case JID_CHAR_SHAPE_BG_BORDER:
                {
                    BorderInfo* borderInfo = NULL;

                    for (int i = 0; i < (int)jsonValue.size(); i++)
                    {
                        borderInfo = new BorderInfo();

                        if (borderInfo)
                        {
                            Json::Value childValue = jsonValue[i];

                            parsingElement(childValue, borderInfo);
                            m_borderInfo.push_back(borderInfo);
                        }
                    }
                    break;
                }
                case JID_CHAR_SHAPE_BG_BORDER_POSITION:
                    if (Obj != NULL)
                    {
                        ((BorderInfo*)Obj)->position = (LinePosition)nValue;
                        ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                    }
                    break;
                case JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE:
                    if (Obj != NULL)
                    {
                        ((BorderInfo*)Obj)->lineShape = (LineShape)nValue;
                        ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                    }
                    break;
                case JID_CHAR_SHAPE_BG_BORDER_SIZE:
                    if (Obj != NULL)
                    {
                        ((BorderInfo*)Obj)->size = nValue;
                        ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                    }
                    break;
                case JID_CHAR_SHAPE_BG_BORDER_COLOR:
                    if (Obj != NULL)
                    {
                        ((BorderInfo*)Obj)->color = nValue;
                        ((BorderInfo*)Obj)->checkedList.push_back(mapIter->second);
                    }
                    break;
                case JID_CHAR_SHAPE_BG_COLOR:
                    m_backgroundColor = nValue;
                    break;
                case JID_CHAR_SHAPE_BG_PATTONCOLOR:
                    m_pattonColor = nValue;
                    break;
                case JID_CHAR_SHAPE_BG_PATTONTYPE:
                    m_pattonType = (PattonType)nValue;
                    break;
                default:
                    bFind = false;
                    break;
                }
            }             

            if (bFind)
            {
                m_checkedList.push_back(mapIter->second);
                m_mapCheckedList.insert(std::map<std::string, int>::value_type(mapIter->first, mapIter->second));
                bFind = false;
            }

            it++;
        }

        return true;
    }
}