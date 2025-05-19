use super::{latency::Latency, percentile::PercentileSpectrum, request_sec::RequestSec, units};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WrkMetrics {
    pub endpoint: String,
    pub threads: u32,
    pub connections: u32,
    pub latency: Latency,
    pub latency_distribution: HashMap<String, f64>,
    pub percentile_spectrum: PercentileSpectrum,
    pub req: RequestSec,
    pub total_requests: u64,
    pub duration: f64,
    pub requests_per_sec: f64,
    pub transfer_per_sec: String,
}

impl Display for WrkMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Endpoint: {}\nThreads: {}\nConnections: {}\nLatency: {:?}\nReq/Sec: {:?}\nTotal Requests: {}\nDuration: {}s\nRequests/sec: {}\nTransfer/sec: {}",
            self.endpoint,
            self.threads,
            self.connections,
            self.latency,
            self.req,
            self.total_requests,
            self.duration,
            self.requests_per_sec,
            self.transfer_per_sec
        )
    }
}

impl From<&str> for WrkMetrics {
    fn from(output: &str) -> Self {
        let lines = output
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();

        let endpoint = lines
            .iter()
            .find(|l| l.starts_with("Running"))
            .and_then(|l| l.split('@').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let (threads, connections) = lines
            .iter()
            .find(|l| l.contains("threads and"))
            .and_then(|l| {
                let mut parts = l.split_whitespace();
                let threads = parts.next()?.parse().ok()?;
                let connections = parts.nth(2)?.parse().ok()?;
                Some((threads, connections))
            })
            .unwrap_or((0, 0));

        let latency = lines
            .iter()
            .find(|l| l.starts_with("Latency"))
            .map(|&l| Latency::from(l))
            .unwrap_or_default();

        let latency_distribution = parse_latency_distribution(&lines);
        let percentile_spectrum = PercentileSpectrum::from(lines.as_slice());

        let req = lines
            .iter()
            .find(|l| l.starts_with("Req/Sec"))
            .map(|&l| RequestSec::from(l))
            .unwrap_or_default();

        let (total_requests, duration) = lines
            .iter()
            .find(|l| l.contains("requests in"))
            .and_then(|l| parse_requests_line(l))
            .unwrap_or((0, 0.0));

        let requests_per_sec = lines
            .iter()
            .find(|l| l.contains("Requests/sec:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| f64::from_str(s).ok())
            .unwrap_or(0.0);

        let transfer_per_sec = lines
            .iter()
            .find(|l| l.contains("Transfer/sec:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .map(String::from)
            .unwrap_or_default();

        WrkMetrics {
            endpoint,
            threads,
            connections,
            latency,
            latency_distribution,
            percentile_spectrum,
            req,
            total_requests,
            duration,
            requests_per_sec,
            transfer_per_sec,
        }
    }
}

fn parse_latency_distribution(lines: &[&str]) -> HashMap<String, f64> {
    match lines
        .iter()
        .position(|l| l.contains("Latency Distribution"))
    {
        Some(idx) => lines
            .iter()
            .skip(idx + 1)
            .take_while(|l| !l.is_empty())
            .filter(|l| l.contains('%'))
            .filter_map(|l| {
                let parts = l.split_whitespace().collect::<Vec<_>>();
                if parts.len() >= 2 {
                    let percent = parts[0].to_string();
                    let value = units::parse_to_milliseconds(parts[1]);
                    Some((percent, value))
                } else {
                    None
                }
            })
            .collect::<HashMap<String, f64>>(),
        None => HashMap::new(),
    }
}

/// Parses the line containing the total requests and duration
/// Returns a tuple of total requests and duration in seconds
fn parse_requests_line(line: &str) -> Option<(u64, f64)> {
    let mut parts = line.split_whitespace();
    let requests = parts.next()?.parse().ok()?;

    let (duration_str, unit): (String, String) = parts
        .nth(2)?
        .trim_end_matches(',')
        .chars()
        .partition(|c| c.is_ascii_digit() || *c == '.');

    if unit.is_empty() {
        // if no unit is provided, assume seconds
        let duration = duration_str.parse().ok()?;
        return Some((requests, duration));
    }

    let duration_num = duration_str.parse::<f64>().ok()?;
    let duration = match unit.trim().to_lowercase().as_str() {
        "h" => duration_num * 3600.0,
        "m" => duration_num * 60.0,
        _ => duration_num,
    };

    Some((requests, duration))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(a: f64, b: f64) {
        const EPSILON: f64 = 1e-6;
        assert!(
            (a - b).abs() < EPSILON,
            "Expected {a} to be approximately equal to {b}"
        );
    }

    const SAMPLE_OUTPUT: &str = r"
Running 10s test @ http://localhost:8080
  2 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   125.12ms   25.31ms 450.00ms   90.12%
    Req/Sec   400.12     50.23   550.00     85.45%
  Latency Distribution
     50%  120.12ms
     75%  130.00ms
     90%  140.23ms
     99%  400.00ms
  8000 requests in 10.00s, 6.42MB read
Requests/sec:    800.12
Transfer/sec:    656.56KB
";

    #[test]
    fn test_wrk_metrics_from() {
        let metrics = WrkMetrics::from(SAMPLE_OUTPUT);
        assert_eq!(metrics.endpoint, "http://localhost:8080");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 125.12);
        assert_float_eq(metrics.latency.stdev, 25.31);
        assert_float_eq(metrics.latency.max, 450.0);
        assert_float_eq(metrics.req.avg, 400.12);
        assert_float_eq(metrics.req.stdev, 50.23);
        assert_float_eq(metrics.req.max, 550.0);
        assert_eq!(metrics.total_requests, 8000);
        assert_float_eq(metrics.duration, 10.0);
        assert_float_eq(metrics.requests_per_sec, 800.12);
        assert_eq!(metrics.transfer_per_sec, "656.56KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50%"], 120.12);
        assert_float_eq(dist["75%"], 130.0);
        assert_float_eq(dist["90%"], 140.23);
        assert_float_eq(dist["99%"], 400.0);
    }

    const SAMPLE_OUTPUT_2: &str = r"
Running 30s test @ http://localhost:8080/index.html
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   635.91us    0.89ms  12.92ms   93.69%
    Req/Sec    56.20k     8.07k   62.00k    86.54%
Latency Distribution
  50% 250.00us
  75% 491.00us
  90% 700.00us
  99% 5.80ms
22464657 requests in 30.00s, 17.76GB read
Requests/sec: 748868.53
Transfer/sec:    606.33MB
";

    #[test]
    fn test_wrk_metrics_from_2() {
        let metrics = WrkMetrics::from(SAMPLE_OUTPUT_2);
        assert_eq!(metrics.endpoint, "http://localhost:8080/index.html");
        assert_eq!(metrics.threads, 12);
        assert_eq!(metrics.connections, 400);
        assert_float_eq(metrics.latency.avg, 0.63591);
        assert_float_eq(metrics.latency.stdev, 0.89);
        assert_float_eq(metrics.latency.max, 12.92);
        assert_float_eq(metrics.req.avg, 56200.0);
        assert_float_eq(metrics.req.stdev, 8070.0);
        assert_float_eq(metrics.req.max, 62000.0);
        assert_eq!(metrics.total_requests, 22_464_657);
        assert_float_eq(metrics.duration, 30.0);
        assert_float_eq(metrics.requests_per_sec, 748_868.53);
        assert_eq!(metrics.transfer_per_sec, "606.33MB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50%"], 0.25);
        assert_float_eq(dist["75%"], 0.491);
        assert_float_eq(dist["90%"], 0.7);
        assert_float_eq(dist["99%"], 5.8);
    }

    const WRK2_INPUT: &str = r"Running 1m test @ http://127.0.0.1:8080/sys/ping
  2 threads and 100 connections
  Thread calibration: mean lat.: 1.473ms, rate sampling interval: 10ms
  Thread calibration: mean lat.: 1.496ms, rate sampling interval: 10ms
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.46ms    2.24ms  44.06ms   98.51%
    Req/Sec     1.05k   265.56     5.40k    89.45%
  Latency Distribution (HdrHistogram - Recorded Latency)
 50.000%    1.18ms
 75.000%    1.54ms
 90.000%    1.93ms
 99.000%   11.72ms
 99.900%   30.74ms
 99.990%   39.52ms
 99.999%   44.03ms
100.000%   44.10ms

  Detailed Percentile spectrum:
       Value   Percentile   TotalCount 1/(1-Percentile)

       0.111     0.000000            1         1.00
       0.642     0.100000         9985         1.11
       0.808     0.200000        19969         1.25
       0.940     0.300000        29880         1.43
       1.060     0.400000        39813         1.67
       1.183     0.500000        49762         2.00
       1.246     0.550000        54786         2.22
       1.312     0.600000        59701         2.50
       1.383     0.650000        64679         2.86
       1.462     0.700000        69689         3.33
       1.545     0.750000        74666         4.00
       1.591     0.775000        77137         4.44
       1.644     0.800000        79634         5.00
       1.703     0.825000        82120         5.71
       1.768     0.850000        84596         6.67
       1.844     0.875000        87067         8.00
       1.887     0.887500        88318         8.89
       1.935     0.900000        89574        10.00
       1.985     0.912500        90819        11.43
       2.046     0.925000        92039        13.33
       2.121     0.937500        93292        16.00
       2.167     0.943750        93911        17.78
       2.215     0.950000        94528        20.00
       2.265     0.956250        95149        22.86
       2.321     0.962500        95782        26.67
       2.381     0.968750        96409        32.00
       2.417     0.971875        96702        35.56
       2.459     0.975000        97020        40.00
       2.517     0.978125        97331        45.71
       2.609     0.981250        97637        53.33
       3.083     0.984375        97946        64.00
       4.923     0.985938        98101        71.11
       7.239     0.987500        98258        80.00
      10.287     0.989062        98412        91.43
      12.351     0.990625        98568       106.67
      15.359     0.992188        98723       128.00
      16.735     0.992969        98802       142.22
      18.175     0.993750        98879       160.00
      19.839     0.994531        98956       182.86
      21.983     0.995313        99034       213.33
      23.423     0.996094        99112       256.00
      24.255     0.996484        99152       284.44
      25.791     0.996875        99190       320.00
      26.479     0.997266        99228       365.71
      27.055     0.997656        99268       426.67
      27.727     0.998047        99306       512.00
      28.127     0.998242        99326       568.89
      28.639     0.998437        99345       640.00
      29.295     0.998633        99366       731.43
      30.047     0.998828        99384       853.33
      30.751     0.999023        99403      1024.00
      31.119     0.999121        99415      1137.78
      31.263     0.999219        99423      1280.00
      31.695     0.999316        99432      1462.86
      32.127     0.999414        99443      1706.67
      32.927     0.999512        99452      2048.00
      33.279     0.999561        99457      2275.56
      33.695     0.999609        99462      2560.00
      34.079     0.999658        99466      2925.71
      34.495     0.999707        99471      3413.33
      34.783     0.999756        99476      4096.00
      35.551     0.999780        99479      4551.11
      36.383     0.999805        99481      5120.00
      38.399     0.999829        99483      5851.43
      38.943     0.999854        99486      6826.67
      39.263     0.999878        99488      8192.00
      39.519     0.999890        99490      9102.22
      39.583     0.999902        99491     10240.00
      42.591     0.999915        99492     11702.86
      42.783     0.999927        99494     13653.33
      42.783     0.999939        99494     16384.00
      42.847     0.999945        99496     18204.44
      42.847     0.999951        99496     20480.00
      42.847     0.999957        99496     23405.71
      43.583     0.999963        99497     27306.67
      43.583     0.999969        99497     32768.00
      43.871     0.999973        99498     36408.89
      43.871     0.999976        99498     40960.00
      43.871     0.999979        99498     46811.43
      44.031     0.999982        99499     54613.33
      44.031     0.999985        99499     65536.00
      44.031     0.999986        99499     72817.78
      44.031     0.999988        99499     81920.00
      44.031     0.999989        99499     93622.86
      44.095     0.999991        99500    109226.67
      44.095     1.000000        99500          inf
#[Mean    =        1.458, StdDeviation   =        2.240]
#[Max     =       44.064, Total count    =        99500]
#[Buckets =           27, SubBuckets     =         2048]
----------------------------------------------------------
  119802 requests in 1.00m, 22.05MB read
Requests/sec:   1996.65
Transfer/sec:    376.32KB";

    #[test]
    fn test_parse_wrk2_output() {
        let metrics = WrkMetrics::from(WRK2_INPUT);
        assert_eq!(metrics.endpoint, "http://127.0.0.1:8080/sys/ping");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 1.46);
        assert_float_eq(metrics.latency.stdev, 2.24);
        assert_float_eq(metrics.latency.max, 44.06);
        assert_float_eq(metrics.req.avg, 1050.0);
        assert_float_eq(metrics.req.stdev, 265.56);
        assert_float_eq(metrics.req.max, 5400.0);
        assert_eq!(metrics.total_requests, 119_802);
        assert_float_eq(metrics.duration, 60.0);
        assert_float_eq(metrics.requests_per_sec, 1996.65);
        assert_eq!(metrics.transfer_per_sec, "376.32KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50.000%"], 1.18);
        assert_float_eq(dist["75.000%"], 1.54);
        assert_float_eq(dist["90.000%"], 1.93);
        assert_float_eq(dist["99.000%"], 11.72);
        assert_float_eq(dist["99.900%"], 30.74);
        assert_float_eq(dist["99.990%"], 39.52);
        assert_float_eq(dist["99.999%"], 44.03);
        assert_float_eq(dist["100.000%"], 44.10);
    }

    const WRK2_INPUT_2: &str = r"
    Running 30s test @ http://127.0.0.1:80/index.html
  2 threads and 100 connections
  Thread calibration: mean lat.: 10087 usec, rate sampling interval: 22 msec
  Thread calibration: mean lat.: 10139 usec, rate sampling interval: 21 msec
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.60ms    1.92ms  12.50ms   68.46%
    Req/Sec     1.04k     1.08k    2.50k    72.79%
  Latency Distribution (HdrHistogram - Recorded Latency)
 50.000%    6.67ms
 75.000%    7.78ms
 90.000%    9.14ms
 99.000%   11.18ms
 99.900%   12.30ms
 99.990%   12.45ms
 99.999%   12.50ms
100.000%   12.50ms

Detailed Percentile spectrum:
     Value   Percentile   TotalCount 1/(1-Percentile)

     0.921     0.000000            1         1.00
     4.053     0.100000         3951         1.11
     4.935     0.200000         7921         1.25
     5.627     0.300000        11858         1.43
     6.179     0.400000        15803         1.67
     6.671     0.500000        19783         2.00
     6.867     0.550000        21737         2.22
     7.079     0.600000        23733         2.50
     7.287     0.650000        25698         2.86
     7.519     0.700000        27659         3.33
     7.783     0.750000        29644         4.00
     7.939     0.775000        30615         4.44
     8.103     0.800000        31604         5.00
     8.271     0.825000        32597         5.71
     8.503     0.850000        33596         6.67
     8.839     0.875000        34571         8.00
     9.015     0.887500        35070         8.89
     9.143     0.900000        35570        10.00
     9.335     0.912500        36046        11.43
     9.575     0.925000        36545        13.33
     9.791     0.937500        37032        16.00
     9.903     0.943750        37280        17.78
    10.015     0.950000        37543        20.00
    10.087     0.956250        37795        22.86
    10.167     0.962500        38034        26.67
    10.279     0.968750        38268        32.00
    10.343     0.971875        38390        35.56
    10.439     0.975000        38516        40.00
    10.535     0.978125        38636        45.71
    10.647     0.981250        38763        53.33
    10.775     0.984375        38884        64.00
    10.887     0.985938        38951        71.11
    11.007     0.987500        39007        80.00
    11.135     0.989062        39070        91.43
    11.207     0.990625        39135       106.67
    11.263     0.992188        39193       128.00
    11.303     0.992969        39226       142.22
    11.335     0.993750        39255       160.00
    11.367     0.994531        39285       182.86
    11.399     0.995313        39319       213.33
    11.431     0.996094        39346       256.00
    11.455     0.996484        39365       284.44
    11.471     0.996875        39379       320.00
    11.495     0.997266        39395       365.71
    11.535     0.997656        39408       426.67
    11.663     0.998047        39423       512.00
    11.703     0.998242        39431       568.89
    11.743     0.998437        39439       640.00
    11.807     0.998633        39447       731.43
    12.271     0.998828        39454       853.33
    12.311     0.999023        39463      1024.00
    12.327     0.999121        39467      1137.78
    12.343     0.999219        39470      1280.00
    12.359     0.999316        39473      1462.86
    12.375     0.999414        39478      1706.67
    12.391     0.999512        39482      2048.00
    12.399     0.999561        39484      2275.56
    12.407     0.999609        39486      2560.00
    12.415     0.999658        39489      2925.71
    12.415     0.999707        39489      3413.33
    12.423     0.999756        39491      4096.00
    12.431     0.999780        39493      4551.11
    12.431     0.999805        39493      5120.00
    12.439     0.999829        39495      5851.43
    12.439     0.999854        39495      6826.67
    12.447     0.999878        39496      8192.00
    12.447     0.999890        39496      9102.22
    12.455     0.999902        39497     10240.00
    12.455     0.999915        39497     11702.86
    12.463     0.999927        39498     13653.33
    12.463     0.999939        39498     16384.00
    12.463     0.999945        39498     18204.44
    12.479     0.999951        39499     20480.00
    12.479     0.999957        39499     23405.71
    12.479     0.999963        39499     27306.67
    12.479     0.999969        39499     32768.00
    12.479     0.999973        39499     36408.89
    12.503     0.999976        39500     40960.00
    12.503     1.000000        39500          inf
#[Mean    =        6.602, StdDeviation   =        1.919]
#[Max     =       12.496, Total count    =        39500]
#[Buckets =           27, SubBuckets     =         2048]
----------------------------------------------------------
60018 requests in 30.00s, 19.81MB read
Requests/sec:   2000.28
Transfer/sec:    676.18KB
";

    #[test]
    fn test_parse_wrk2_output_2() {
        let metrics = WrkMetrics::from(WRK2_INPUT_2);
        assert_eq!(metrics.endpoint, "http://127.0.0.1:80/index.html");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 6.60);
        assert_float_eq(metrics.latency.stdev, 1.92);
        assert_float_eq(metrics.latency.max, 12.50);
        assert_float_eq(metrics.req.avg, 1040.0);
        assert_float_eq(metrics.req.stdev, 1080.0);
        assert_float_eq(metrics.req.max, 2500.0);
        assert_eq!(metrics.total_requests, 60018);
        assert_float_eq(metrics.duration, 30.0);
        assert_float_eq(metrics.requests_per_sec, 2000.28);
        assert_eq!(metrics.transfer_per_sec, "676.18KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50.000%"], 6.67);
        assert_float_eq(dist["75.000%"], 7.78);
        assert_float_eq(dist["90.000%"], 9.14);
        assert_float_eq(dist["99.000%"], 11.18);
        assert_float_eq(dist["99.900%"], 12.30);
        assert_float_eq(dist["99.990%"], 12.45);
        assert_float_eq(dist["99.999%"], 12.50);
        assert_float_eq(dist["100.000%"], 12.50);
    }

    #[test]
    fn test_error_handling() {
        let empty = WrkMetrics::from("invalid output");
        assert_eq!(empty.endpoint, "");
        assert_eq!(empty.threads, 0);
        assert_eq!(empty.connections, 0);
        assert_float_eq(empty.latency.avg, 0.0);
        assert_float_eq(empty.latency.stdev, 0.0);
        assert_float_eq(empty.latency.max, 0.0);
        assert_float_eq(empty.req.avg, 0.0);
        assert_float_eq(empty.req.stdev, 0.0);
        assert_float_eq(empty.req.max, 0.0);
        assert_eq!(empty.total_requests, 0);
        assert_float_eq(empty.duration, 0.0);
        assert_float_eq(empty.requests_per_sec, 0.0);
        assert_eq!(empty.transfer_per_sec, "");
        assert!(empty.latency_distribution.is_empty());
        assert_eq!(empty.percentile_spectrum.percentiles.len(), 0);
    }
}
