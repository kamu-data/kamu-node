import {Component, OnInit} from '@angular/core';
import {Observable, of, throwError} from 'rxjs';
import {environment} from "../../../environments/environment";
import {HttpClient} from "@angular/common/http";
import {catchError, map} from "rxjs/operators";
import {ActivatedRoute, Router} from "@angular/router";

@Component({
  selector: 'app-github-callback',
  templateUrl: './github-callback.component.html'
})

export class GithubCallbackComponent implements OnInit {

  constructor(private route: ActivatedRoute,
              private router: Router,
              private httpClient: HttpClient) {
  }

  public accessToken: any;

  public getToken(code: string) {
    return this.httpClient.post(`https://github.com/login/oauth/access_token?client_id=${environment.github_client_id}&redirect_uri=${environment.redirect_uri}&client_secret=${environment.github_client_secret_id}&code=${code}`,{}).pipe(map((res) => {
        debugger
        // @ts-ignore
        let json = res.json();
            // @ts-ignore
            if (json && json.token) {
                this.accessToken = json;
                localStorage.setItem("access_token", this.accessToken.token);
                return {"authenticated": true};
            } else {
                localStorage.removeItem("access_token");
                return {"authenticated": false};
            }
    }),
        catchError((error: Response) => GithubCallbackComponent.handleError(error) ));
  }

  private static handleError(error: Response): Observable<never> {
      return throwError(`GitHub ${error.statusText || 'Server error'}`);
  }

  ngOnInit() {
      debugger
      setTimeout(() => {
          debugger
          this.route.queryParams.subscribe(
              (param: any) => {
                  debugger
                  let code = param['code'];
                  this.getToken(code).subscribe(() => {
                      debugger
                      return this.router.navigate(['/']);
                  }, (err: string) => {
                      debugger
                      window.alert(err)
                  });
              })
      }, 2000);
  }

}