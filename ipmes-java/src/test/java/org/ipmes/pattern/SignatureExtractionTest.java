package org.ipmes.pattern;

import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.*;

public class SignatureExtractionTest {
    @Test
    public void nodeSignature() {
        SigExtractor extractor = new SimPatternExtractor();
        assertEquals(
                "Process::hello.sh",
                extractor.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Process\",\"subtype\":\"process\",\"name\":\"hello.sh\"}}"))
        );
        assertEquals(
                "Artifact::file::/etc/password",
                extractor.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Artifact\",\"subtype\":\"file\",\"path\":\"/etc/password\"}}"))
        );
        assertEquals(
                "Artifact::network socket::10.2.2.1:8787",
                extractor.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Artifact\",\"subtype\":\"network socket\",\"remote address\":\"10.2.2.1\",\"remote port\":\"8787\"}}"))
        );
        assertEquals(
                "Unknown::",
                extractor.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Unknown\"}}"))
        );
    }
}
