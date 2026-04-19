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
#include "CParaShape.h"

#include "JsonModel.h"

static std::map<std::string, int> parashapeMapData = {
    {JIN_PARA_SHAPE,                          JID_PARA_SHAPE},
    {JIN_PARA_SHAPE_HORIZONTAL,                  JID_PARA_SHAPE_HORIZONTAL},
    {JIN_PARA_SHAPE_MARGINLEFT,                  JID_PARA_SHAPE_MARGINLEFT},
    {JIN_PARA_SHAPE_MARGINRIGHT,              JID_PARA_SHAPE_MARGINRIGHT},
    {JIN_PARA_SHAPE_FIRSTLINE,                  JID_PARA_SHAPE_FIRSTLINE},
    {JIN_PARA_SHAPE_INDENT,                      JID_PARA_SHAPE_INDENT},
    {JIN_PARA_SHAPE_OUTDENT,                  JID_PARA_SHAPE_OUTDENT},
    {JIN_PARA_SHAPE_LINESPACING,              JID_PARA_SHAPE_LINESPACING},
    {JIN_PARA_SHAPE_LINESPACINGVALUE,          JID_PARA_SHAPE_LINESPACINGVALUE},
    {JIN_PARA_SHAPE_SPACINGPARAUP,              JID_PARA_SHAPE_SPACINGPARAUP},
    {JIN_PARA_SHAPE_SPACINGPARABOTTOM,          JID_PARA_SHAPE_SPACINGPARABOTTOM},
    {JIN_PARA_SHAPE_SPACINGGRIDPAPER,          JID_PARA_SHAPE_SPACINGGRIDPAPER},
    {JIN_PARA_SHAPE_LINEBREAKKOREAN,          JID_PARA_SHAPE_LINEBREAKKOREAN},
    {JIN_PARA_SHAPE_LINEBREAKENGLISH,          JID_PARA_SHAPE_LINEBREAKENGLISH},
    {JIN_PARA_SHAPE_LINEBREAKCONDENSE,          JID_PARA_SHAPE_LINEBREAKCONDENSE},
    {JIN_PARA_SHAPE_PARATYPE,                  JID_PARA_SHAPE_PARATYPE},
    {JIN_PARA_SHAPE_PARATYPEVALUE,              JID_PARA_SHAPE_PARATYPEVALUE},
    {JIN_PARA_SHAPE_WIDOWORPHAN,              JID_PARA_SHAPE_WIDOWORPHAN},
    {JIN_PARA_SHAPE_KEEPWITHNEXT,              JID_PARA_SHAPE_KEEPWITHNEXT},
    {JIN_PARA_SHAPE_KEEPLINESTOGETHER,          JID_PARA_SHAPE_KEEPLINESTOGETHER},
    {JIN_PARA_SHAPE_PAGEBREAKBEFORE,          JID_PARA_SHAPE_PAGEBREAKBEFORE},
    {JIN_PARA_SHAPE_FONTLINEHEIGHT,              JID_PARA_SHAPE_FONTLINEHEIGHT},
    {JIN_PARA_SHAPE_LINEWRAP,                  JID_PARA_SHAPE_LINEWRAP},
    {JIN_PARA_SHAPE_AUTOSPACEEASIANENG,          JID_PARA_SHAPE_AUTOSPACEEASIANENG},
    {JIN_PARA_SHAPE_AUTOSPACEEASIANNUM,          JID_PARA_SHAPE_AUTOSPACEEASIANNUM},
    {JIN_PARA_SHAPE_VERTICALALIGN,              JID_PARA_SHAPE_VERTICALALIGN},
    {JIN_PARA_SHAPE_TABTYPES,                  JID_PARA_SHAPE_TABTYPES},
    {JIN_PARA_SHAPE_TABTYPE,                  JID_PARA_SHAPE_TABTYPE},
    {JIN_PARA_SHAPE_TABSHAPE,                  JID_PARA_SHAPE_TABSHAPE},
    {JIN_PARA_SHAPE_TABPOSITION,              JID_PARA_SHAPE_TABPOSITION},
    {JIN_PARA_SHAPE_AUTOTABINDENT,              JID_PARA_SHAPE_AUTOTABINDENT},
    {JIN_PARA_SHAPE_AUTOTABPARARIGHTEND,      JID_PARA_SHAPE_AUTOTABPARARIGHTEND},
    {JIN_PARA_SHAPE_BASETABSPACE,              JID_PARA_SHAPE_BASETABSPACE},
    {JIN_PARA_SHAPE_BORDER,                      JID_PARA_SHAPE_BORDER},
    {JIN_PARA_SHAPE_BORDERPOSITION,              JID_PARA_SHAPE_BORDERPOSITION},
    {JIN_PARA_SHAPE_BORDERTYPE,                  JID_PARA_SHAPE_BORDERTYPE},
    {JIN_PARA_SHAPE_BORDERSIZE,                  JID_PARA_SHAPE_BORDERSIZE},
    {JIN_PARA_SHAPE_BORDERCOLOR,              JID_PARA_SHAPE_BORDERCOLOR},
    {JIN_PARA_SHAPE_BGCOLOR,                  JID_PARA_SHAPE_BGCOLOR},
    {JIN_PARA_SHAPE_BGPATTONCOLOR,              JID_PARA_SHAPE_BGPATTONCOLOR},
    {JIN_PARA_SHAPE_BGPATTONTYPE,              JID_PARA_SHAPE_BGPATTONTYPE},
    {JIN_PARA_SHAPE_SPACINGLEFT,              JID_PARA_SHAPE_SPACINGLEFT},
    {JIN_PARA_SHAPE_SPACINGRIGHT,              JID_PARA_SHAPE_SPACINGRIGHT},
    {JIN_PARA_SHAPE_SPACINGTOP,                  JID_PARA_SHAPE_SPACINGTOP},
    {JIN_PARA_SHAPE_SPACINGBOTTOM,              JID_PARA_SHAPE_SPACINGBOTTOM},
    {JIN_PARA_SHAPE_SPACINGIGNORE,              JID_PARA_SHAPE_SPACINGIGNORE}
};

namespace CheckListModule {

    CParaShape::CParaShape(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);        
    }

    CParaShape::CParaShape(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CParaShape::~CParaShape()
    {
        if (m_tabTypes.size() > 0)
        {
            std::vector<TabTypes*>::iterator iter = m_tabTypes.begin();

            for (iter; iter != m_tabTypes.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_tabTypes.clear();
        }

        if (m_borderInfo.size() > 0)
        {
            std::vector<BorderInfo*>::iterator iter2 = m_borderInfo.begin();

            for (iter2; iter2 != m_borderInfo.end(); iter2++)
            {
                if ((*iter2))
                    delete *iter2;
                *iter2 = NULL;
            }
            m_borderInfo.clear();
        }
    }

    HAlignType CParaShape::getHorizontal()
    {
        return m_horizontal;
    }

    JsonINT CParaShape::getMarginLeft()
    {
        return m_marginLeft;
    }

    JsonINT CParaShape::getMarginRight()
    {
        return m_marginRight;
    }


    bool CParaShape::getFirstline()
    {
        return m_Firstline;
    }

    JsonUINT CParaShape::getIndent()
    {
        return m_indent;
    }

    JsonUINT CParaShape::getOutdent()
    {
        return m_outdent;
    }

    LineSpacingType    CParaShape::getLinespacing()
    {
        return m_linespacing;
    }

    JsonUINT CParaShape::getLinespacingvalue()
    {
        return m_linespacingvalue;
    }

    JsonUINT CParaShape::getSpacingParaup()
    {
        return m_spacingParaup;
    }

    JsonUINT CParaShape::getSpacingParabottom()
    {
        return m_spacingParabottom;
    }

    bool CParaShape::getSpacingGridpaper()
    {
        return m_spacingGridpaper;
    }

    bool CParaShape::getLinebreakKorean()
    {
        return m_linebreakKorean;
    }

    LineBreakLatinWord CParaShape::getLinebreakEnglish()
    {
        return m_linebreakEnglish;
    }

    JsonUINT CParaShape::getLinebreakCondense()
    {
        return m_linebreakCondense;
    }


    ParaType CParaShape::getParaType()
    {
        return m_paraType;
    }

    UINT CParaShape::getParatypeValue()
    {
        return m_paratypeValue;
    }


    bool CParaShape::getWidowOrphan()
    {
        return m_widowOrphan;
    }

    bool CParaShape::getKeppWithNext()
    {
        return m_keepWithNext;
    }

    bool CParaShape::getKeepLinesTogether()
    {
        return m_keepLinesTogether;
    }

    bool CParaShape::getPageBreakBefore()
    {
        return m_pageBreakBefore;
    }

    bool CParaShape::getFontLineHeight()
    {
        return m_fontLineHeight;
    }

    bool CParaShape::getLineWrap()
    {
        return m_lineWrap;
    }

    bool CParaShape::getAutoSpaceEasianEng()
    {
        return m_autoSpaceEasianEng;
    }

    bool CParaShape::getAutoSpaceEasianNum()
    {
        return m_autoSpaceEasianNum;
    }

    VAlignType CParaShape::getVerticalAlign()
    {
        return m_verticalAlign;
    }

    std::vector<TabTypes*>    CParaShape::getTabTypes(TabType type)
    {
        return m_tabTypes;
    }

    bool CParaShape::getAutotabIntent()
    {
        return m_autotabIntent;
    }

    bool CParaShape::getAutotabPararightEnd()
    {
        return m_autotabPararightEnd;
    }

    JsonUINT CParaShape::getBaseTabSpace()
    {
        return m_baseTabSpace;
    }


    BorderInfo* CParaShape::getBorderInfo(LinePosition pos)
    {
        int i = 0;
        for (i = 0; i < m_borderInfo.size(); i++)
        {
            if (m_borderInfo.at(i)->position == pos)
                return m_borderInfo.at(i);
        }
        return NULL;
    }

    UINT CParaShape::getBackgroundColor()
    {
        return m_bgColor;
    }

    UINT CParaShape::getPattonColor()
    {
        return m_bgPattonColor;
    }

    PattonType CParaShape::getPattonType()
    {
        return m_bgPattonType;
    }


    bool CParaShape::getSpacingLeft()
    {
        return m_spacingLeft;
    }

    bool CParaShape::getSpacingRight()
    {
        return m_spacingRight;
    }

    bool CParaShape::getSpacingTop()
    {
        return m_spacingTop;
    }

    bool CParaShape::getSpacingBottom()
    {
        return m_spacingBottom;
    }

    bool CParaShape::getSpacingIgnore()
    {
        return m_spacingIgnore;
    }


    bool CParaShape::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_PARA_SHAPE];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CParaShape::parsingDVCDocument(LPCSTR document)
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

    bool CParaShape::parsingElement(Json::Value root, void* Obj)
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

            mapIter = parashapeMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_PARA_SHAPE:
                break;
            case JID_PARA_SHAPE_HORIZONTAL:
                m_horizontal = (HAlignType)nValue;
                break;
            case JID_PARA_SHAPE_MARGINLEFT:
                if (root[it.name().c_str()].isInt())
                    m_marginLeft.value = nValue;
                else {
                    m_marginLeft.min = jsonValue[JNI_INT_MIN].asInt();
                    m_marginLeft.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_marginLeft.min == m_marginLeft.max)
                        m_marginLeft.value = m_marginLeft.min;
                }
                break;
            case JID_PARA_SHAPE_MARGINRIGHT:
                if (root[it.name().c_str()].isInt())
                    m_marginRight.value = nValue;
                else {
                    m_marginRight.min = jsonValue[JNI_INT_MIN].asInt();
                    m_marginRight.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_marginRight.min == m_marginRight.max)
                        m_marginRight.value = m_marginRight.min;
                }
                break;
            case JID_PARA_SHAPE_FIRSTLINE:
                m_Firstline = bValue;
                break;
            case JID_PARA_SHAPE_INDENT:
                if (root[it.name().c_str()].isInt())
                    m_indent.value = nValue;
                else {
                    m_indent.min = jsonValue[JNI_INT_MIN].asInt();
                    m_indent.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_indent.min == m_indent.max)
                        m_indent.value = m_indent.min;
                }
                break;
            case JID_PARA_SHAPE_OUTDENT:
                if (root[it.name().c_str()].isInt())
                    m_outdent.value = nValue;
                else {
                    m_outdent.min = jsonValue[JNI_INT_MIN].asInt();
                    m_outdent.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_outdent.min == m_outdent.max)
                        m_outdent.value = m_outdent.min;
                }
                break;
            case JID_PARA_SHAPE_LINESPACING:
                m_linespacing = (LineSpacingType)nValue;
                break;
            case JID_PARA_SHAPE_LINESPACINGVALUE:
                if (root[it.name().c_str()].isInt())
                    m_linespacingvalue.value = nValue;
                else {
                    m_linespacingvalue.min = jsonValue[JNI_INT_MIN].asInt();
                    m_linespacingvalue.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_linespacingvalue.min == m_linespacingvalue.max)
                        m_linespacingvalue.value = m_linespacingvalue.min;
                }
                break;
            case JID_PARA_SHAPE_SPACINGPARAUP:
                if (root[it.name().c_str()].isInt())
                    m_spacingParaup.value = nValue;
                else {
                    m_spacingParaup.min = jsonValue[JNI_INT_MIN].asInt();
                    m_spacingParaup.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_spacingParaup.min == m_spacingParaup.max)
                        m_spacingParaup.value = m_spacingParaup.min;
                }
                break;
            case JID_PARA_SHAPE_SPACINGPARABOTTOM:
                if (root[it.name().c_str()].isInt())
                    m_spacingParabottom.value = nValue;
                else {
                    m_spacingParabottom.min = jsonValue[JNI_INT_MIN].asInt();
                    m_spacingParabottom.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_spacingParabottom.min == m_spacingParabottom.max)
                        m_spacingParabottom.value = m_spacingParabottom.min;
                }
                break;
            case JID_PARA_SHAPE_SPACINGGRIDPAPER:
                m_spacingGridpaper = bValue;
                break;
            case JID_PARA_SHAPE_LINEBREAKKOREAN:
                m_linebreakKorean = bValue;
                break;
            case JID_PARA_SHAPE_LINEBREAKENGLISH:
                m_linebreakEnglish = (LineBreakLatinWord)nValue;
                break;
            case JID_PARA_SHAPE_LINEBREAKCONDENSE:
                if (root[it.name().c_str()].isInt())
                    m_linebreakCondense.value = nValue;
                else {
                    m_linebreakCondense.min = jsonValue[JNI_INT_MIN].asInt();
                    m_linebreakCondense.max = jsonValue[JNI_INT_MAX].asInt();
                    if (m_linebreakCondense.min == m_linebreakCondense.max)
                        m_linebreakCondense.value = m_linebreakCondense.min;
                }
                break;
            case JID_PARA_SHAPE_PARATYPE:
                m_paraType = (ParaType)nValue;
                break;
            case JID_PARA_SHAPE_PARATYPEVALUE:
                m_paratypeValue = nValue;
                break;
            case JID_PARA_SHAPE_WIDOWORPHAN:
                m_widowOrphan = bValue;
                break;
            case JID_PARA_SHAPE_KEEPWITHNEXT:
                m_keepWithNext = bValue;
                break;
            case JID_PARA_SHAPE_KEEPLINESTOGETHER:
                m_keepLinesTogether = bValue;
                break;
            case JID_PARA_SHAPE_PAGEBREAKBEFORE:
                m_pageBreakBefore = bValue;
                break;
            case JID_PARA_SHAPE_FONTLINEHEIGHT:
                m_fontLineHeight = bValue;
                break;
            case JID_PARA_SHAPE_LINEWRAP:
                m_lineWrap = bValue;
                break;
            case JID_PARA_SHAPE_AUTOSPACEEASIANENG:
                m_autoSpaceEasianEng = bValue;
                break;
            case JID_PARA_SHAPE_AUTOSPACEEASIANNUM:
                m_autoSpaceEasianNum = bValue;
                break;
            case JID_PARA_SHAPE_VERTICALALIGN:
                m_verticalAlign = (VAlignType)nValue;
                break;
            case JID_PARA_SHAPE_TABTYPES:
            {
                TabTypes* tabTypes = NULL;

                for (int i = 0; i < (int)jsonValue.size(); i++)
                {
                    tabTypes = new TabTypes();

                    if (tabTypes)
                    {
                        Json::Value childValue = jsonValue[i];
                        Json::ValueIterator its = childValue.begin();

                        int nValue1 = 0;
                        bool bValue1 = false;
                        std::string strValue1 = "";
                        Json::Value jsonValue1;
                        std::map<std::string, int>::iterator mapIter1;

                        parsingElement(childValue, tabTypes);

                        m_tabTypes.push_back(tabTypes);
                    }
                }
                break;
            }
            case JID_PARA_SHAPE_TABTYPE:
                if (Obj != NULL)
                {
                    ((TabTypes*)Obj)->tabType = (TabType)nValue;
                    ((TabTypes*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_PARA_SHAPE_TABSHAPE:
                if (Obj != NULL)
                {
                    ((TabTypes*)Obj)->tabShape = (LineShape)nValue;
                    ((TabTypes*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_PARA_SHAPE_TABPOSITION:
                if (Obj != NULL)
                {
                    ((TabTypes*)Obj)->position = nValue;
                    ((TabTypes*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_PARA_SHAPE_AUTOTABINDENT:
                m_autotabIntent;
                break;
            case JID_PARA_SHAPE_AUTOTABPARARIGHTEND:
                m_autotabPararightEnd;
                break;
            case JID_PARA_SHAPE_BASETABSPACE:
                m_baseTabSpace;
                break;
            case JID_PARA_SHAPE_BORDER:
                m_borderInfo;
                break;
            case JID_PARA_SHAPE_BORDERPOSITION:
                break;
            case JID_PARA_SHAPE_BORDERTYPE:
                break;
            case JID_PARA_SHAPE_BORDERSIZE:
                break;
            case JID_PARA_SHAPE_BORDERCOLOR:
                break;
            case JID_PARA_SHAPE_BGCOLOR:
                m_bgColor;
                break;
            case JID_PARA_SHAPE_BGPATTONCOLOR:
                m_bgPattonColor;
                break;
            case JID_PARA_SHAPE_BGPATTONTYPE:
                m_bgPattonType;
                break;
            case JID_PARA_SHAPE_SPACINGLEFT:
                m_spacingLeft;
                break;
            case JID_PARA_SHAPE_SPACINGRIGHT:
                m_spacingRight;
                break;
            case JID_PARA_SHAPE_SPACINGTOP:
                m_spacingTop;
                break;
            case JID_PARA_SHAPE_SPACINGBOTTOM:
                m_spacingBottom;
                break;
            case JID_PARA_SHAPE_SPACINGIGNORE:
                m_spacingIgnore;
                break;
            default:
                bFind = false;
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