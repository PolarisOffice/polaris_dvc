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

#include "DVCInterface.h"

#include "CCharShape.h"
#include "CParaShape.h"
#include "CTable.h"
#include "CBullet.h"
#include "CHyperlink.h"
#include "COutlineShape.h"
#include "CParaNumBullet.h"
#include "CStyle.h"
#include "CSpecialCharacter.h"
#include "CMacro.h"

class CheckList
{
public:
    CheckList(std::string file);
    ~CheckList();

public:
    bool parsing();
    
    ICheckListObj* getModel(DVCType modelType);

    
private:    
    std::string m_file;
    std::vector<ICheckListObj*>  m_objList;
    

};

