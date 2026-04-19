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
#include "CBullet.h"


static std::map<std::string, int> bulletMapData = {
    {JIN_BULLET,                        JID_BULLET},
    {JIN_BULLET_TYPE,                    JID_BULLET_TYPE},
    {JIN_BULLET_CHECKTYPE,                JID_BULLET_CHECKTYPE},
    {JIN_BULLET_CODE,                    JID_BULLET_CODE},
    {JIN_BULLET_SHAPES,                    JID_BULLET_SHAPES}
};

namespace CheckListModule {

    CBullet::CBullet(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    CBullet::CBullet(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CBullet::~CBullet()
    {

    }

    BulletType CBullet::getBulletType()
    {
        return m_bulletType;
    }

    CheckType CBullet::getCheckType()
    {
        return m_checkType;
    }

    std::vector<UINT> CBullet::getCodes()
    {
        return m_codes;
    }

    std::string CBullet::getBulletShapes()
    {
        return m_bulletShapes;
    }

    bool CBullet::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_BULLET];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CBullet::parsingDVCDocument(LPCSTR document)
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

    bool CBullet::parsingElement(Json::Value root, void* Obj)
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

            mapIter = bulletMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_BULLET_TYPE:
                m_bulletType = (BulletType)nValue;
                break;
            case JID_BULLET_CHECKTYPE:
                m_checkType = (CheckType)nValue;
                break;
            case JID_BULLET_CODE:
                for (int i = 0; i < (int)jsonValue.size(); i++)
                {
                    UINT code = jsonValue[i].asInt();
                    m_codes.push_back(code);
                }
                break;
            case JID_BULLET_SHAPES:
                m_bulletShapes = strValue;
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