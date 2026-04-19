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
#include "CParaNumBullet.h"

static std::map<std::string, int> paranumBulletMapData = {
    {JIN_PARANUMBULLET,                            JID_PARANUMBULLET},
    {JIN_PARANUMBULLET_TYPE,                    JID_PARANUMBULLET_TYPE},
    {JIN_PARANUMBULLET_STARTNUMBER,                JID_PARANUMBULLET_STARTNUMBER},
    {JIN_PARANUMBULLET_VALUE,                    JID_PARANUMBULLET_VALUE},
    {JIN_PARANUMBULLET_LEVELTYPE,                JID_PARANUMBULLET_LEVELTYPE},
    {JIN_PARANUMBULLET_LEVELTYPE_LEVEL,            JID_PARANUMBULLET_LEVELTYPE_LEVEL},
    {JIN_PARANUMBULLET_LEVELTYPE_NUMBERTYPE,    JID_PARANUMBULLET_LEVELTYPE_NUMBERTYPE},
    {JIN_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE,    JID_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE}
};

namespace CheckListModule {

    CParaNumBullet::CParaNumBullet(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    CParaNumBullet::CParaNumBullet(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CParaNumBullet::~CParaNumBullet()
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

    ParaNumType CParaNumBullet::getParaNumType()
    {
        return m_paraNumType;
    }

    bool CParaNumBullet::getStartNumber()
    {
        return m_startNumber;
    }

    UINT CParaNumBullet::getValue()
    {
        return m_value;
    }

    std::vector<LevelType*> CParaNumBullet::getLevelTypes()
    {
        return m_levelTypes;
    }

    bool CParaNumBullet::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_PARANUMBULLET];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CParaNumBullet::parsingDVCDocument(LPCSTR document)
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

    bool CParaNumBullet::parsingElement(Json::Value root, void* Obj)
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

            mapIter = paranumBulletMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_PARANUMBULLET_TYPE:
                m_paraNumType = (ParaNumType)nValue;
                break;
            case JID_PARANUMBULLET_STARTNUMBER:
                m_startNumber = bValue;
                break;
            case JID_PARANUMBULLET_VALUE:
                m_value = nValue;
                break;
            case JID_PARANUMBULLET_LEVELTYPE:
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
            case JID_PARANUMBULLET_LEVELTYPE_LEVEL:
                if (Obj != NULL)
                {
                    ((LevelType*)Obj)->level = nValue;
                    ((LevelType*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_PARANUMBULLET_LEVELTYPE_NUMBERTYPE:
                if (Obj != NULL)
                {
                    ((LevelType*)Obj)->numberType = strValue;
                    ((LevelType*)Obj)->checkedList.push_back(mapIter->second);
                }
                break;
            case JID_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE:
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
