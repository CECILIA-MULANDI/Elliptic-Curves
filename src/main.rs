use std::env;
use itertools::Itertools;
#[derive(Debug,Hash,Eq,PartialEq,Clone)]
struct Point{
    x:u128,
    y:u128
}
#[derive(Debug)]
struct EllipticCurve {
    a: u128,
    b: u128,
}
fn is_prime(n:u128)->bool{
    if n < 2{
        return false;
    }
    let limit = (n as f64).sqrt() as u128;
    for i in 2..=limit{
        if n%i == 0{
            return false
        }
    }
    true
}
fn is_on_curve(point:&Point, curve: &EllipticCurve,finite_field:u128)->bool{
    let left_side = (point.y * point.y) % finite_field;
    let right_side = (point.x.pow(3) + curve.a * point.x + curve.b) % finite_field;
    left_side == right_side
}

fn generate_curve(curve: &EllipticCurve, finite_field: u128) -> Vec<Point> {
    let mut points = Vec::new();
    for x in 0..finite_field {
        // Calculate right side carefully to avoid overflow
        let x_squared = (x * x) % finite_field;
        let x_cubed = (x_squared * x) % finite_field;
        let ax = (curve.a * x) % finite_field;
        let right_side = (x_cubed + ax + curve.b) % finite_field;
        
        for y in 0..finite_field {
            let y_squared = (y * y) % finite_field;
            if y_squared == right_side {
                points.push(Point { x, y });
                if y != 0 {  // Only add symmetric point if y != 0
                    points.push(Point { x, y: finite_field - y });
                }
            }
        }
    }
    points.into_iter().unique().collect()
}
fn point_addition(p: Point, q: Point, curve: &EllipticCurve, finite_field: u128) -> Option<Point> {
    if p != q {
        // Point addition (P ≠ Q)
        let dy = if q.y >= p.y {
            q.y - p.y
        } else {
            finite_field - (p.y - q.y)
        };
        
        let dx = if q.x >= p.x {
            q.x - p.x
        } else {
            finite_field - (p.x - q.x)
        };
        
        if dx == 0 {
            return None;
        }
        
        let inverse_denominator = mod_inverse(dx, finite_field)?;
        let slope = (dy * inverse_denominator) % finite_field;
        
        // Calculate x3 using safe arithmetic
        let slope_squared = (slope * slope) % finite_field;
        let x3 = if slope_squared >= (p.x + q.x) {
            slope_squared - p.x - q.x
        } else {
            finite_field - ((p.x + q.x - slope_squared) % finite_field)
        };
        let x3 = x3 % finite_field;
        
        // Calculate y3 using safe arithmetic
        let term1 = if p.x >= x3 {
            slope * (p.x - x3)
        } else {
            slope * (finite_field - (x3 - p.x))
        };
        let term1 = term1 % finite_field;
        
        let y3 = if term1 >= p.y {
            term1 - p.y
        } else {
            finite_field - (p.y - term1)
        };
        let y3 = y3 % finite_field;
        
        Some(Point { x: x3, y: y3 })
    } else {
        // Point doubling (P = P)
        if p.y == 0 {
            return None;  // Tangent line is vertical
        }
        
        // Calculate slope = (3x² + a) / (2y)
        let x_squared = (p.x * p.x) % finite_field;
        let numerator = (3 * x_squared + curve.a) % finite_field;
        let denominator = (2 * p.y) % finite_field;
        
        let inverse_denominator = mod_inverse(denominator, finite_field)?;
        let slope = (numerator * inverse_denominator) % finite_field;
        
        // Calculate x3 = slope² - 2x
        let slope_squared = (slope * slope) % finite_field;
        let x3 = if slope_squared >= (2 * p.x) {
            slope_squared - 2 * p.x
        } else {
            finite_field - ((2 * p.x - slope_squared) % finite_field)
        };
        let x3 = x3 % finite_field;
        
        // Calculate y3 = slope(x - x3) - y
        let term1 = if p.x >= x3 {
            slope * (p.x - x3)
        } else {
            slope * (finite_field - (x3 - p.x))
        };
        let term1 = term1 % finite_field;
        
        let y3 = if term1 >= p.y {
            term1 - p.y
        } else {
            finite_field - (p.y - term1)
        };
        let y3 = y3 % finite_field;
        
        Some(Point { x: x3, y: y3 })
    }
}
fn extended_euclidean(a: u128, b: u128) -> (u128, i128, i128) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (gcd, x1, y1) = extended_euclidean(b, a % b);
        let x = y1;
        let y = x1 - ((a / b) as i128) * y1;
        (gcd, x, y)
    }
}
fn mod_inverse(a: u128, p: u128) -> Option<u128> {
    let (gcd, x, _) = extended_euclidean(a, p);
    if gcd == 1 {
        // Convert negative x to positive modulo p
        Some(((x % p as i128 + p as i128) % p as i128) as u128)
    } else {
        None
    }
}
fn main(){
    let args:Vec<String> = env::args().collect();

    if args.len() != 4{
        eprintln!("Usage: {} <finite_field> <a> <b>", args[0]);
        return;
    }
    let finite_field = &args[1];
    let a = &args[2];
    let b = &args[3];

        
    match finite_field.trim().parse::<u128>(){
        Ok(num)=>{
            if is_prime(num){
                println!("{num} is the finite field you have chosen");
                    // Parse a and b for the elliptic curve
                let a_coeff = a.trim().parse::<u128>().unwrap_or(0);
                let b_coeff = b.trim().parse::<u128>().unwrap_or(0);

                if a_coeff > 0 && a_coeff < num && b_coeff > 0 && b_coeff < num {
                    let curve = EllipticCurve { a: a_coeff, b: b_coeff };
                    println!("Elliptic Curve parameters: {:?}", curve);
                
                    let point_p = Point { x: 2130, y: 2999 };
                    let point_q = Point { x: 8592, y: 2572 };
                    if is_on_curve(&point_p, &curve, num) && is_on_curve(&point_q, &curve, num){
                        match point_addition(point_p, point_q, &curve, num){
                            Some(result) => println!("The result is: {:?}", result),
                            None => println!("Point addition resulted in None."),
                        }
                    }
                    else{
                        println!("One or both points are not on the curve.");
                    }
                }
                else {
                    println!("The values of a and b cannot be greater than {finite_field}");
                }
            }
            else{
                println!("{num} is not prime");
            }
        }
        Err(_)=>{
            println!("The input is invalid");
        }
    }

}