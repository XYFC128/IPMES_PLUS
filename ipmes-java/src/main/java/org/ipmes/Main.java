package org.ipmes;

import java.io.BufferedReader;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.stream.Collectors;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.stream.input.InputHandler;
import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.impl.Arguments;
import net.sourceforge.argparse4j.inf.ArgumentParser;
import net.sourceforge.argparse4j.inf.ArgumentParserException;
import net.sourceforge.argparse4j.inf.Namespace;
import org.ipmes.decomposition.TCQGenerator;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.join.Join;
import org.ipmes.join.NaiveJoin;
import org.ipmes.join.PriorityJoin;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.*;
import org.ipmes.siddhi.TCQueryOutputCallback;
import org.ipmes.siddhi.TCSiddhiAppGenerator;

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
        String ttpPrefix = ns.getString("pattern_prefix");
        String dataGraphPath = ns.getString("data_graph");
        Long windowSize = ns.getLong("windowSize");

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
                        extractor).get();
        DependencyGraph temporalPattern = DependencyGraph.parse(new FileReader(orelsFile)).get();

        MatchResult.MAX_NUM_EDGES = spatialPattern.numEdges();
        MatchResult.MAX_NUM_NODES = spatialPattern.numNodes();

        System.out.println("Patterns:");
        spatialPattern.getEdges().forEach(System.out::println);

        // Decomposition
        TCQGenerator d = new TCQGenerator(temporalPattern, spatialPattern);
        ArrayList<TCQuery> tcQueries = d.decompose();
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(spatialPattern, temporalPattern, tcQueries);
        gen.setUseRegex(useRegex);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        SiddhiManager siddhiManager = new SiddhiManager();
        SiddhiAppRuntime runtime = siddhiManager.createSiddhiAppRuntime(appStr);
        Join join = new PriorityJoin(temporalPattern, spatialPattern, windowSize * 1000, tcQueries);
        for (TCQuery q : tcQueries) {
            runtime.addCallback(
                    String.format("TC%dOutput", q.getId()),
                    new TCQueryOutputCallback(q, spatialPattern, join));
        }

        BufferedReader inputReader = new BufferedReader(new FileReader(dataGraphPath));
        String line = inputReader.readLine();
        InputHandler inputHandler = runtime.getInputHandler("InputStream");
        runtime.start();

        EventSorter sorter = new EventSorter(tcQueries, useRegex);
        EventSender sender = new EventSender(inputHandler, sorter);
        while (line != null) {
            sender.sendLine(line);
            line = inputReader.readLine();
        }
        sender.flushBuffers();

        System.out.println("Match Results:");
        ArrayList<ArrayList<MatchEdge>> results = join.extractAnswer();
        for (ArrayList<MatchEdge> result : results) {
            System.out.print("[");
            System.out.print(
                    result.stream()
                            .map(edge -> Long.toString(edge.getDataId()))
                            .collect(Collectors.joining(",")));
            System.out.println("]");
        }

        runtime.shutdown();
        siddhiManager.shutdown();
    }
}