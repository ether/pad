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
import("fileutils");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");

import("etherpad.utils.*");
import("etherpad.globals.*");
import("etherpad.log");
import("fastJSON");

jimport("java.io.File",
        "java.io.DataInputStream", 
        "java.io.FileInputStream",
        "java.lang.Byte",
        "java.io.FileReader",
        "java.io.BufferedReader",
        "java.security.MessageDigest",
        "java.lang.Runtime",
        "net.appjet.common.util.BetterFile",
	"java.lang.ProcessBuilder",
	"java.lang.Process",
	"java.io.InputStreamReader"
	);


function getImageSize(filename, page) {
  var proc;
  if (filename.split(".").pop().toLowerCase() == 'pdf') {
    proc = ProcessBuilder("src/plugins/imageConvert/identifyImage.sh",
			  filename, page).start();
  } else {
    proc = ProcessBuilder("identify",
			  "-format",
			  "%[fx:w]\n%[fx:h]",
			  filename + "[" + page + "]").start();
  }
  var procStdout = BufferedReader(new InputStreamReader(proc.getInputStream()));
  var w = parseFloat(procStdout.readLine());
  var h = parseFloat(procStdout.readLine());
  proc.waitFor();
  return {w:w, h:h}
}

function convertImage(inFileName, page, outFileName, offset, size, pixelSize) {
  var proc;
  if (inFileName.split(".").pop().toLowerCase() == 'pdf') {
    var dpix = pixelSize.w * 72.0 / size.w;
    var dpiy = pixelSize.h * 72.0 / size.h;
    proc = ProcessBuilder("src/plugins/imageConvert/convertImage.sh",
			  inFileName,
			  outFileName,
			  page,
			  dpix, dpiy,
			  offset.x, offset.y,
			  offset.x + size.w, offset.y + size.h);
  } else {
    proc = ProcessBuilder("convert",
			  "-crop",
			  "" + size.w + "x" + size.h + "+" + offset.x + "+" + offset.y,
			  "-scale",
			  "" + pixelSize.w + "x" + pixelSize.w,
			  inFileName + "["+page+"]",
			  outFileName);
  }
  proc.start().waitFor();
}

function onRequest() {
  var path = "src/plugins/fileUpload/upload/" + request.path.toString().slice("/ep/imageConvert/".length);  

  if (request.params.action == "getSize") {
    var imageSize = getImageSize(path, 0);
    response.setContentType("text/plain");
    response.write(fastJSON.stringify(imageSize));
  } else {
    var page = request.params.p === undefined ? 0 : parseInt(request.params.p);
    var offset = {x:(request.params.x === undefined) ? 0 : parseInt(request.params.x),
		  y:(request.params.y === undefined) ? 0 : parseInt(request.params.y)};
    var size = {w:(request.params.w === undefined) ? 0 : parseInt(request.params.w),
		h:(request.params.h === undefined) ? 0 : parseInt(request.params.h)};
    var pixelSize = {w:(request.params.pw === undefined) ? 0 : parseInt(request.params.pw),
		     h:(request.params.ph === undefined) ? 0 : parseInt(request.params.ph)};

    var outFileName = path.split(".");
    var extension = outFileName.pop();
    outFileName.push("" + page + ":" + offset.x + "," +  offset.y + ":" + size.w + "," +  size.h + ":" + pixelSize.w + "," +  pixelSize.h);
    outFileName.push("png");
    outFileName = outFileName.join(".");

    convertImage(path, page, outFileName, offset, size, pixelSize);

    response.setContentType("image/png");

    var file = FileInputStream(File(outFileName));
    response.writeBytes(BetterFile.getStreamBytes(file));
    file.close();

  }

  if (request.acceptsGzip) {
    response.setGzip(true);
  }
  return true;
}
