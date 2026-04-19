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
#include "CStyle.h"


static std::map<std::string, int> styleMapData = {
    {JIN_STYLE,                            JID_STYLE},
    {JIN_STYLE_TYPE,                    JID_STYLE_TYPE},
    {JIN_STYLE_PERMISSION,                JID_STYLE_PERMISSION}
};

namespace CheckListModule {

    CStyle::CStyle(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    CStyle::CStyle(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CStyle::~CStyle()
    {

    }

    StyleType CStyle::getStyleType()
    {
        return m_styleType;
    }

    bool CStyle::getPermission()
    {
        return m_permission;
    }

    bool CStyle::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_STYLE];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CStyle::parsingDVCDocument(LPCSTR document)
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

    bool CStyle::parsingElement(Json::Value root, void* Obj)
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

            mapIter = styleMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
             case JID_STYLE:
                 break;
             case JID_STYLE_TYPE:
                 m_styleType = (StyleType)nValue;
                 break;
             case JID_STYLE_PERMISSION:
                 m_permission = bValue;
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