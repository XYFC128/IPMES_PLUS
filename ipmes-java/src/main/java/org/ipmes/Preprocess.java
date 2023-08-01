package org.ipmes;

import io.siddhi.core.event.Event;
import org.json.*;

public class Preprocess {
    public static String extractNodeSignature(JSONObject nodeObj) {
        JSONObject properties = nodeObj.getJSONObject("properties");
        String type = properties.getString("type");
        String signature = String.format("%s::", type);
        if (type.equals("Process")) {
            signature += properties.getString("name");
        } else if (type.equals("Artifact")) {
            String subtype = properties.getString("subtype");
            signature += String.format("%s::", subtype);
            if (subtype.equals("file") || subtype.equals("directory")) {
                signature += properties.getString("path");
            } else if (subtype.equals("network socket")) {
                signature += String.format(
                        "%s:%s",
                        properties.getString("remote address"),
                        properties.getString("remote port"));
            }
        }
        return signature;
    }

    public static String extractEdgeSignature(JSONObject edgeObj) {
        return edgeObj.getJSONObject("properties").getString("operation");
    }

    public static String extractTimestamp(JSONObject edgeObj) {
        return edgeObj.getJSONObject("properties").getString("lastest");
    }

    public static Object[] toEventData(String eventStr) {
        JSONObject inpObj = new JSONObject(eventStr);
        JSONObject edgeObj = inpObj.getJSONObject("r");
        String ts = extractTimestamp(edgeObj);
        String eid = edgeObj.getString("id");
        String esig = extractEdgeSignature(edgeObj);
        String startId = inpObj.getJSONObject("m").getString("id");
        String startSig = extractNodeSignature(inpObj.getJSONObject("m"));
        String endId = inpObj.getJSONObject("n").getString("id");
        String endSig = extractNodeSignature(inpObj.getJSONObject("n"));

        return new Object[]{ts, eid, esig, startId, startSig, endId, endSig};
    }
}
