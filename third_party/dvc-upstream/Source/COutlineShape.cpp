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
#include "COutlineShape.h"

static std::map<std::string, int> outlineShapeMapData = {    
    {JIN_OUTLINESHAPE,                            JID_OUTLINESHAPE},
    {JIN_OUTLINESHAPE_TYPE,                        JID_OUTLINESHAPE_TYPE},
    {JIN_OUTLINESHAPE_STARTNUMBER,                JID_OUTLINESHAPE_STARTNUMBER},
    {JIN_OUTLINESHAPE_VALUE,                    JID_OUTLINESHAPE_VALUE},
    {JIN_OUTLINESHAPE_LEVELTYPE,                JID_OUTLINESHAPE_LEVELTYPE},
    {JIN_OUTLINESHAPE_LEVELTYPE_LEVEL,            JID_OUTLINESHAPE_LEVELTYPE_LEVEL},
    {JIN_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE,        JID_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE},
    {JIN_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE,    JID_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE}
};

namespace CheckListModule {

    COutlineShape::COutlineShape(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    COutlineShape::COutlineShape(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    COutlineShape::~COutlineShape()
    {
        if (m_levelTypes.size() > 0)
        {
            std::vector<LevelType*>::iterator iter = m_levelTypes.begin();

            for (iter; iter != m_levelTypes.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_levelTypes.clear();
        }
    }

    OutlineType COutlineShape::getOutlineType()
    {
        return m_outlineType;
    }

    bool COutlineShape::getStartNumber()
    {
        return m_startNumber;
    }

    UINT COutlineShape::getValue()
    {
        return m_value;
    }

    std::vector<LevelType*> COutlineShape::getLevelTypes()
    {
        return m_levelTypes;
    }

    bool COutlineShape::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_OUTLINESHAPE];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool COutlineShape::parsingDVCDocument(LPCSTR document)
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

    bool COutlineShape::parsingElement(Json::Value root, void* Obj)
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

            mapIter = outlineShapeMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_OUTLINESHAPE_TYPE:
                m_outlineType = (OutlineType)nValue;
                break;
            case JID_OUTLINESHAPE_STARTNUMBER:
                m_startNumber = bValue;
                break;
            case JID_OUTLINESHAPE_VALUE:
                m_value = nValue;
                break;
            case JID_OUTLINESHAPE_LEVELTYPE:
            {
                LevelType* levelType = NULL;

                for (int i = 0; i < (int)jsonValue.size(); i++)
                {
                    levelType = new LevelType();

                    if (levelType)
                    {
                        Json::Value childValue = jsonValue[i];

                        parsingElement(childValue, levelType);
                        m_levelTypes.push_back(levelType);
                    }
                }
                break;
            }
            case JID_OUTLINESHAPE_LEVELTYPE_LEVEL:
                if (Obj != NULL)
                {
                    ((LevelType*)Obj)->level = nValue;
                    ((LevelType*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE:
                if (Obj != NULL)
                {
                    ((LevelType*)Obj)->numberType = strValue;
                    ((LevelType*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE:
                if (Obj != NULL)
                {
                    ((LevelType*)Obj)->numShapeType = (NumberShapeType)nValue;
                    ((LevelType*)Obj)->checkedList.push_back(mapIter->second);
                }
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
