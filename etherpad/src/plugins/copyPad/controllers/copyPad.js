/**
 * Copyright 2009 RedHog, Egil Möller <egil.moller@piratpartiet.se>
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *      http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import("faststatic");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");

import("etherpad.utils.*");
import("etherpad.collab.server_utils");
import("etherpad.globals.*");
import("etherpad.log");
import("etherpad.pad.padusers");
import("etherpad.pro.pro_utils");
import("etherpad.pro.pro_pad_db");
import("etherpad.helpers");
import("etherpad.pro.pro_accounts.getSessionProAccount");
import("sqlbase.sqlbase");
import("sqlbase.sqlcommon");
import("sqlbase.sqlobj");
import("etherpad.pad.padutils");
import("etherpad.pad.model");
import("etherpad.collab.server_utils");
import("etherpad.collab.collab_server.buildHistoricalAuthorDataMapForPadHistory");
import("etherpad.collab.ace.easysync2.{Changeset,AttribPool}");

function formatAuthorData(historicalAuthorData) {
  var authors_all = [];
  for (var author in historicalAuthorData) {
    var n = historicalAuthorData[author].name;
    authors_all[n] = (authors_all[n]) ? 1+authors_all[n] : 1;
  }
  
  var authors = [];
  for (var n in authors_all) {
   if (n == "undefined") {
     authors.push("[unnamed author]");
   } else {
     authors.push(n);
   }
  }
  return authors;
}

function createCopy(localPadId, pad, clonePadId, cloneRevNum) {
  if (pad.exists()) {
    throw new Error("Destination pad already exists");
  }

  var usePadId;
  if (server_utils.isReadOnlyId(clonePadId)) {
    usePadId = server_utils.readonlyToPadId(clonePadId);
    if (!pro_utils.isProDomainRequest()) {
      usePadId = padutils.getGlobalPadId(usePadId);
    }
  } else { 
    usePadId = padutils.getGlobalPadId(clonePadId);
  }
 
  var cloneData = model.accessPadGlobal(usePadId, function(pad) {
    if (cloneRevNum == undefined)
      cloneRevNum = pad.getHeadRevisionNumber();

    return {
      'padText':pad.getRevisionText(cloneRevNum),
      'padAText': pad.getInternalRevisionAText(cloneRevNum),
      'pool': pad.pool(),
      'historicalAuthorData': buildHistoricalAuthorDataMapForPadHistory(pad)
    };
  }, 'r');

  var author_list = formatAuthorData(cloneData.historicalAuthorData);
  var header = "This pad builds on [["+clonePadId+"/rev."+cloneRevNum + "]], created by " + author_list.join(" & ") + "\n\n";

  pad.create('');

  var pool = pad.pool();
  pool.fromJsonable(cloneData.pool.toJsonable());
  var assem = Changeset.smartOpAssembler();
  assem.appendOpWithText('+', header, [], pool);
  Changeset.appendATextToAssembler(cloneData.padAText, assem);
  assem.endDocument();
  pad.appendRevision(Changeset.pack(1, header.length + cloneData.padText.length + 1, assem.toString(), header + cloneData.padText));

  return;
}

function onRequest() {
  if (request.params['old'] == undefined) {
    throw new Error("No source pad specified");
  }

  if (request.params['new'] != undefined) {
    padId = padutils.makeValidLocalPadId(request.params['new']);
  } else {
    if (pro_utils.isProDomainRequest()) {
      padId = pro_pad_db.getNextPadId();
    } else {
      padId = padutils.globalToLocalId(randomUniquePadId());
    }
  }

  padutils.accessPadLocal(padId, function(pad) {
    createCopy(padId, pad, request.params.old, request.params.old_rev);
    response.redirect('/'+ padId);
  });
  return true;
}
