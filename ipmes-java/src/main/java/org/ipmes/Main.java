package org.ipmes;

import java.io.BufferedReader;
import java.io.FileReader;
import java.util.*;

import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.impl.Arguments;
import net.sourceforge.argparse4j.inf.ArgumentParser;
import net.sourceforge.argparse4j.inf.ArgumentParserException;
import net.sourceforge.argparse4j.inf.Namespace;

import org.ipmes.decomposition.TCQGenerator;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.join.Join;
import org.ipmes.join.NaiveJoin;
import org.ipmes.join.PriorityJoin;
import org.ipmes.match.FullMatch;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.LightMatchResult;
import org.ipmes.pattern.*;

import org.json.JSONObject;

public class Main {

    static ArgumentParser getParser() {
        ArgumentParser parser = ArgumentParsers.newFor("ipmes-java").build()
                .defaultHelp(true)
                .description("IPMES implemented in Java.");
        parser.addArgument("-r", "--regex")
                .dest("useRegex")
                .setDefault(false)
                .action(Arguments.storeTrue())
                .help("Explicitly use regex matching. Default will automatically depend on the pattern prefix name");
        parser.addArgument("--darpa")
                .action(Arguments.storeTrue())
                .setDefault(false)
                .help("We are running on DARPA dataset.");
        parser.addArgument("-w", "--window-size").type(Long.class)
                .dest("windowSize")
                .setDefault(1800L)
                .help("Time window size (sec) when joining.");
        parser.addArgument("--debug")
                .action(Arguments.storeTrue())
                .setDefault(false)
                .help("Output debug information.");
        parser.addArgument("pattern_prefix").type(String.class)
                .required(true)
                .help("The path prefix of pattern's files, e.g. ./data/patterns/TTP11");
        parser.addArgument("data_graph").type(String.class)
                .required(true)
                .help("The path to the preprocessed data graph");

        return parser;
    }

    public static void main(String[] args) throws Exception {
        // parse argument
        ArgumentParser parser = getParser();
        Namespace ns = null;
        try {
            ns = parser.parseArgs(args);
        } catch (ArgumentParserException e) {
            parser.handleError(e);
            System.exit(1);
        }
        Boolean useRegex = ns.getBoolean("useRegex");
        Boolean isDarpa = ns.getBoolean("darpa");
        Boolean isDebug = ns.getBoolean("debug");
        String ttpPrefix = ns.getString("pattern_prefix");
        String dataGraphPath = ns.getString("data_graph");
        long windowSize = ns.getLong("windowSize") * 1000;

        // parse data
        String orelsFile;
        if (ttpPrefix.endsWith("regex")) {
            useRegex = true;
            orelsFile = ttpPrefix.substring(0, ttpPrefix.length() - 5) + "oRels.json";
        } else {
            orelsFile = ttpPrefix + "_oRels.json";
        }

        SigExtractor extractor;
        if (isDarpa)
            extractor = new DarpaExtractor();
        else
            extractor = new SimPatternExtractor();
        PatternGraph spatialPattern = PatternGraph
                .parse(
                        new FileReader(ttpPrefix + "_node.json"),
                        new FileReader(ttpPrefix + "_edge.json"),
                        extractor)
                .get();
        TemporalRelation temporalPattern = TemporalRelation.parse(new FileReader(orelsFile)).get();

        LightMatchResult.MAX_NUM_NODES = spatialPattern.numNodes();

        if (isDebug) {
            System.err.println("Patterns:");
            spatialPattern.getEdges().forEach(System.err::println);
        }

        // Decomposition
        TCQGenerator d = new TCQGenerator(temporalPattern, spatialPattern);
        ArrayList<TCQuery> tcQueries = d.decompose();

        Join join = new PriorityJoin(temporalPattern, spatialPattern, windowSize, tcQueries);

        BufferedReader inputReader = new BufferedReader(new FileReader(dataGraphPath));
        String line = inputReader.readLine();

        TCMatcher matcher = new TCMatcher(tcQueries, useRegex, windowSize, join);
        EventSender sender = new EventSender(matcher);
        while (line != null) {
            sender.sendLine(line);
            line = inputReader.readLine();
        }
        sender.flushBuffers();

        // output
        JSONObject output = new JSONObject();

        output.put("PeekPoolSize", join.getPeakPoolSize());

        Integer[] usageCount = join.getUsageCount();
        Map<Integer, Integer> usageCountMap = new HashMap<>();
        for (int i = 0; i < usageCount.length; i++) {
            int len = tcQueries.get(i).numEdges();
            int count = usageCountMap.getOrDefault(tcQueries.get(i).numEdges(), 0);
            usageCountMap.put(len, count + usageCount[i]);
        }
        output.put("UsageCount", usageCountMap);

        Collection<FullMatch> results = join.extractAnswer();
        output.put("NumResults", results.size());
        if (isDebug) {
            System.err.println("Match Results:");
            for (FullMatch result : results)
                System.err.println(result);
        }

        System.out.print(output.toString(2));
    }
}